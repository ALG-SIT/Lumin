import Foundation
import Security

final class ModelDownloadClient: NSObject, URLSessionDownloadDelegate, @unchecked Sendable {
    private let progressHandler: @Sendable (Double) -> Void
    private var continuation: CheckedContinuation<URL, Error>?
    private var destination: URL?
    private var completionError: Error?
    private var task: URLSessionDownloadTask?
    private lazy var session = URLSession(configuration: .default, delegate: self, delegateQueue: nil)

    init(progress: @escaping @Sendable (Double) -> Void) {
        progressHandler = progress
    }

    func download(_ request: URLRequest, to destination: URL) async throws -> URL {
        self.destination = destination
        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                self.continuation = continuation
                let task = session.downloadTask(with: request)
                self.task = task
                task.resume()
            }
        } onCancel: {
            self.task?.cancel()
        }
    }

    func cancel() {
        task?.cancel()
    }

    func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didWriteData bytesWritten: Int64,
        totalBytesWritten: Int64,
        totalBytesExpectedToWrite: Int64
    ) {
        guard totalBytesExpectedToWrite > 0 else { return }
        progressHandler(Double(totalBytesWritten) / Double(totalBytesExpectedToWrite))
    }

    func urlSession(
        _ session: URLSession,
        downloadTask: URLSessionDownloadTask,
        didFinishDownloadingTo location: URL
    ) {
        guard let response = downloadTask.response as? HTTPURLResponse,
              (200...299).contains(response.statusCode) else {
            completionError = ModelDownloadError.httpStatus(
                (downloadTask.response as? HTTPURLResponse)?.statusCode ?? -1
            )
            return
        }
        guard let destination else {
            completionError = ModelDownloadError.missingDestination
            return
        }
        do {
            try? FileManager.default.removeItem(at: destination)
            try FileManager.default.moveItem(at: location, to: destination)
        } catch {
            completionError = error
        }
    }

    func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        didCompleteWithError error: Error?
    ) {
        guard let continuation else { return }
        self.continuation = nil
        self.task = nil
        session.finishTasksAndInvalidate()
        if let error = completionError ?? error {
            continuation.resume(throwing: error)
        } else if let destination {
            continuation.resume(returning: destination)
        } else {
            continuation.resume(throwing: ModelDownloadError.missingDestination)
        }
    }
}

enum ModelDownloadError: LocalizedError {
    case httpStatus(Int)
    case missingDestination
    case insufficientStorage(required: Int64)

    var errorDescription: String? {
        switch self {
        case .httpStatus(401), .httpStatus(403):
            "Hugging Faceの利用許諾またはトークンを確認してください。"
        case .httpStatus(let status):
            "モデルのダウンロードに失敗しました（HTTP \(status)）。"
        case .missingDestination:
            "モデルの保存先を作成できませんでした。"
        case .insufficientStorage(let required):
            "空き容量が不足しています。約\(ByteCountFormatter.string(fromByteCount: required, countStyle: .file))必要です。"
        }
    }
}

enum HuggingFaceCredentialStore {
    private static let service = "jp.lumin.learning.huggingface"
    private static let account = "access-token"

    static func load() -> String {
        var query = baseQuery
        query[kSecReturnData as String] = true
        query[kSecMatchLimit as String] = kSecMatchLimitOne
        var result: CFTypeRef?
        guard SecItemCopyMatching(query as CFDictionary, &result) == errSecSuccess,
              let data = result as? Data,
              let token = String(data: data, encoding: .utf8) else { return "" }
        return token
    }

    static func save(_ token: String) throws {
        let normalized = token.trimmingCharacters(in: .whitespacesAndNewlines)
        if normalized.isEmpty {
            SecItemDelete(baseQuery as CFDictionary)
            return
        }
        let data = Data(normalized.utf8)
        let status = SecItemUpdate(
            baseQuery as CFDictionary,
            [kSecValueData as String: data] as CFDictionary
        )
        if status == errSecItemNotFound {
            var query = baseQuery
            query[kSecValueData as String] = data
            let addStatus = SecItemAdd(query as CFDictionary, nil)
            guard addStatus == errSecSuccess else { throw KeychainError.status(addStatus) }
        } else if status != errSecSuccess {
            throw KeychainError.status(status)
        }
    }

    private static var baseQuery: [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        ]
    }
}

private enum KeychainError: LocalizedError {
    case status(OSStatus)

    var errorDescription: String? {
        "トークンをKeychainへ保存できませんでした。"
    }
}
