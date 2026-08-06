import Foundation
import MultipeerConnectivity

@MainActor
final class LocalPeerService: NSObject, ObservableObject {
    enum Mode { case idle, teacher, student }

    @Published private(set) var mode: Mode = .idle
    @Published private(set) var connectedPeers: [String] = []
    @Published private(set) var nearbyTeachers: [MCPeerID] = []
    @Published private(set) var statusText = "未接続"
    @Published private(set) var pendingAnalysisCount = 0
    @Published private(set) var joinCode = ""

    var onSessionReceived: ((LearningSession) -> Void)?
    var onSessionEnded: ((UUID) -> Void)?
    var onQuizReceived: ((Quiz) -> Void)?
    var onAnalysisReceived: ((AnalysisEvent) -> Void)?

    private let serviceType = "lumin-class"
    private var peerID = MCPeerID(displayName: "Lumin-\(String(UUID().uuidString.prefix(4)))")
    private lazy var session = MCSession(peer: peerID, securityIdentity: nil, encryptionPreference: .required)
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?
    private var pendingAnalyses: [UUID: AnalysisEvent] = [:]
    private let pendingStorageKey = "lumin.pending-analysis.v1"

    override init() {
        super.init()
        session.delegate = self
        restorePendingAnalyses()
    }

    func startHosting() {
        stop()
        mode = .teacher
        joinCode = String(format: "%04d", Int.random(in: 0...9_999))
        advertiser = MCNearbyServiceAdvertiser(peer: peerID, discoveryInfo: ["app": "Lumin"], serviceType: serviceType)
        advertiser?.delegate = self
        advertiser?.startAdvertisingPeer()
        statusText = "参加を受付中"
    }

    func startBrowsing() {
        stop()
        mode = .student
        browser = MCNearbyServiceBrowser(peer: peerID, serviceType: serviceType)
        browser?.delegate = self
        browser?.startBrowsingForPeers()
        statusText = "教室を検索中"
    }

    func connect(to teacher: MCPeerID, joinCode: String) {
        let normalizedCode = joinCode.trimmingCharacters(in: .whitespacesAndNewlines)
        guard normalizedCode.count == 4,
              normalizedCode.allSatisfy(\.isNumber) else {
            statusText = "4桁の参加コードを入力してください"
            return
        }
        browser?.invitePeer(
            teacher,
            to: session,
            withContext: Data(normalizedCode.utf8),
            timeout: 12
        )
        statusText = "接続しています"
    }

    func sendQuiz(_ quiz: Quiz) {
        send(.quiz(quiz))
    }

    func sendSession(_ learningSession: LearningSession) {
        send(.session(learningSession))
    }

    func sendSessionEnded(_ sessionID: UUID) {
        send(.sessionEnded(sessionID))
    }

    func sendAnalysis(_ analysis: AnalysisEvent) {
        pendingAnalyses[analysis.id] = analysis
        persistPendingAnalyses()
        flushPendingAnalyses()
    }

    func stop() {
        advertiser?.stopAdvertisingPeer()
        browser?.stopBrowsingForPeers()
        advertiser = nil
        browser = nil
        session.disconnect()
        nearbyTeachers = []
        connectedPeers = []
        mode = .idle
        joinCode = ""
        statusText = "未接続"
    }

    private func send(_ message: PeerMessage, to peers: [MCPeerID]? = nil) {
        let recipients = peers ?? session.connectedPeers
        guard !recipients.isEmpty,
              let data = try? JSONEncoder().encode(message) else { return }
        try? session.send(data, toPeers: recipients, with: .reliable)
    }

    private func flushPendingAnalyses() {
        guard mode == .student, !session.connectedPeers.isEmpty else { return }
        for event in pendingAnalyses.values.sorted(by: { $0.submittedAt < $1.submittedAt }) {
            send(.analysis(event))
        }
    }

    private func acknowledge(_ eventID: UUID) {
        guard pendingAnalyses.removeValue(forKey: eventID) != nil else { return }
        persistPendingAnalyses()
    }

    private func persistPendingAnalyses() {
        pendingAnalysisCount = pendingAnalyses.count
        guard let data = try? JSONEncoder().encode(Array(pendingAnalyses.values)) else { return }
        UserDefaults.standard.set(data, forKey: pendingStorageKey)
    }

    private func restorePendingAnalyses() {
        guard let data = UserDefaults.standard.data(forKey: pendingStorageKey),
              let events = try? JSONDecoder().decode([AnalysisEvent].self, from: data) else { return }
        pendingAnalyses = Dictionary(uniqueKeysWithValues: events.map { ($0.id, $0) })
        pendingAnalysisCount = pendingAnalyses.count
    }

    private func refreshConnectionState() {
        connectedPeers = session.connectedPeers.map(\.displayName)
        if connectedPeers.isEmpty {
            statusText = mode == .teacher ? "参加を受付中" : "教室を検索中"
        } else {
            statusText = "\(connectedPeers.count)台 接続中"
            flushPendingAnalyses()
        }
    }
}

extension LocalPeerService: MCSessionDelegate {
    nonisolated func session(_ session: MCSession, peer peerID: MCPeerID, didChange state: MCSessionState) {
        Task { @MainActor in self.refreshConnectionState() }
    }

    nonisolated func session(_ session: MCSession, didReceive data: Data, fromPeer peerID: MCPeerID) {
        guard let message = try? JSONDecoder().decode(PeerMessage.self, from: data) else { return }
        Task { @MainActor in
            switch message {
            case .session(let learningSession): self.onSessionReceived?(learningSession)
            case .sessionEnded(let sessionID): self.onSessionEnded?(sessionID)
            case .quiz(let quiz): self.onQuizReceived?(quiz)
            case .analysis(let event):
                self.onAnalysisReceived?(event)
                self.send(.acknowledgment(event.id), to: [peerID])
            case .acknowledgment(let eventID): self.acknowledge(eventID)
            }
        }
    }

    nonisolated func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) {}
    nonisolated func session(_ session: MCSession, didStartReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, with progress: Progress) {}
    nonisolated func session(_ session: MCSession, didFinishReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, at localURL: URL?, withError error: Error?) {}
}

extension LocalPeerService: MCNearbyServiceAdvertiserDelegate {
    nonisolated func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didReceiveInvitationFromPeer peerID: MCPeerID, withContext context: Data?, invitationHandler: @escaping (Bool, MCSession?) -> Void) {
        Task { @MainActor in
            let submittedCode = context.flatMap { String(data: $0, encoding: .utf8) }
            let accepted = !self.joinCode.isEmpty && submittedCode == self.joinCode
            invitationHandler(accepted, accepted ? self.session : nil)
        }
    }

    nonisolated func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didNotStartAdvertisingPeer error: Error) {
        Task { @MainActor in self.statusText = "受付を開始できませんでした" }
    }
}

extension LocalPeerService: MCNearbyServiceBrowserDelegate {
    nonisolated func browser(_ browser: MCNearbyServiceBrowser, foundPeer peerID: MCPeerID, withDiscoveryInfo info: [String: String]?) {
        Task { @MainActor in
            guard !self.nearbyTeachers.contains(peerID) else { return }
            self.nearbyTeachers.append(peerID)
        }
    }

    nonisolated func browser(_ browser: MCNearbyServiceBrowser, lostPeer peerID: MCPeerID) {
        Task { @MainActor in self.nearbyTeachers.removeAll { $0 == peerID } }
    }

    nonisolated func browser(_ browser: MCNearbyServiceBrowser, didNotStartBrowsingForPeers error: Error) {
        Task { @MainActor in self.statusText = "検索を開始できませんでした" }
    }
}
