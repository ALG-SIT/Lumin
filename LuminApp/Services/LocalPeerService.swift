import Foundation
import MultipeerConnectivity

@MainActor
final class LocalPeerService: NSObject, ObservableObject {
    enum Mode { case idle, teacher, student }

    @Published private(set) var mode: Mode = .idle
    @Published private(set) var connectedPeers: [String] = []
    @Published private(set) var nearbyTeachers: [MCPeerID] = []
    @Published private(set) var statusText = "未接続"

    var onQuizReceived: ((Quiz) -> Void)?
    var onAnalysisReceived: ((AnalysisEvent) -> Void)?

    private let serviceType = "lumin-class"
    private var peerID = MCPeerID(displayName: "Lumin-\(String(UUID().uuidString.prefix(4)))")
    private lazy var session = MCSession(peer: peerID, securityIdentity: nil, encryptionPreference: .required)
    private var advertiser: MCNearbyServiceAdvertiser?
    private var browser: MCNearbyServiceBrowser?

    override init() {
        super.init()
        session.delegate = self
    }

    func startHosting() {
        stop()
        mode = .teacher
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

    func connect(to teacher: MCPeerID) {
        browser?.invitePeer(teacher, to: session, withContext: nil, timeout: 12)
        statusText = "接続しています"
    }

    func sendQuiz(_ quiz: Quiz) {
        send(.quiz(quiz))
    }

    func sendAnalysis(_ analysis: AnalysisEvent) {
        send(.analysis(analysis))
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
        statusText = "未接続"
    }

    private func send(_ message: PeerMessage) {
        guard !session.connectedPeers.isEmpty,
              let data = try? JSONEncoder().encode(message) else { return }
        try? session.send(data, toPeers: session.connectedPeers, with: .reliable)
    }

    private func refreshConnectionState() {
        connectedPeers = session.connectedPeers.map(\.displayName)
        if connectedPeers.isEmpty {
            statusText = mode == .teacher ? "参加を受付中" : "教室を検索中"
        } else {
            statusText = "\(connectedPeers.count)台 接続中"
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
            case .quiz(let quiz): self.onQuizReceived?(quiz)
            case .analysis(let event): self.onAnalysisReceived?(event)
            }
        }
    }

    nonisolated func session(_ session: MCSession, didReceive stream: InputStream, withName streamName: String, fromPeer peerID: MCPeerID) {}
    nonisolated func session(_ session: MCSession, didStartReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, with progress: Progress) {}
    nonisolated func session(_ session: MCSession, didFinishReceivingResourceWithName resourceName: String, fromPeer peerID: MCPeerID, at localURL: URL?, withError error: Error?) {}
}

extension LocalPeerService: MCNearbyServiceAdvertiserDelegate {
    nonisolated func advertiser(_ advertiser: MCNearbyServiceAdvertiser, didReceiveInvitationFromPeer peerID: MCPeerID, withContext context: Data?, invitationHandler: @escaping (Bool, MCSession?) -> Void) {
        Task { @MainActor in invitationHandler(true, self.session) }
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
