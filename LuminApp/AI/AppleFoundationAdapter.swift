import Foundation

#if canImport(FoundationModels)
import FoundationModels

@available(iOS 26.0, *)
@Generable
struct AppleGuidanceOutput {
    @Guide(description: "プロンプトに示された誤概念候補コードを一つ")
    var misconception: String

    @Guide(description: "正解そのものを含めない、短い日本語のヒント")
    var hint: String
}

@available(iOS 26.0, *)
@Generable
struct AppleLessonPlanOutput {
    @Guide(description: "授業で扱う焦点")
    var focus: String

    @Guide(description: "合計10分の授業手順", .count(4))
    var steps: [String]

    @Guide(description: "理解を確かめる短い問題")
    var checkQuestion: String

    @Guide(description: "教師が確認・修正するときの注意点")
    var teacherNote: String
}
#endif
