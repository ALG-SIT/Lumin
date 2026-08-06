import Foundation

public extension SampleData {
    static let geometryAndProbability = Quiz(
        id: "geometry-probability-01",
        title: "図形と確率 ミニチェック",
        subject: "中学数学",
        topic: "図形・確率",
        questions: [
            QuizQuestion(
                id: "math-gp-01",
                prompt: "公平なさいころを1回投げるとき、偶数が出る確率は？",
                concept: "確率",
                acceptedAnswers: ["1/2", "0.5", "50%"],
                misconceptionAnswers: ["1/3": "偶数の個数の数え間違い", "2/3": "有利な場合と全場合の取り違え"],
                genericMisconception: "確率の分母・分子の理解不足",
                hints: ["出る目を1から6まで書き出そう。", "偶数は2・4・6の3通りです。", "確率は、有利な場合の数÷すべての場合の数です。"],
                explanation: "偶数は6通り中3通りなので、3/6=1/2です。"
            ),
            QuizQuestion(
                id: "math-gp-02",
                prompt: "直角をはさむ2辺が3cmと4cmの直角三角形。斜辺の長さは？",
                concept: "三平方の定理",
                acceptedAnswers: ["5", "5cm"],
                misconceptionAnswers: ["7": "辺の長さをそのまま加算", "25": "平方根を取り忘れている"],
                genericMisconception: "三平方の定理の適用不足",
                hints: ["斜辺をcとして a²+b²=c² を使おう。", "3²+4²を計算しよう。", "c²=25。最後にcを求めよう。"],
                explanation: "3²+4²=9+16=25なので、斜辺は√25=5cmです。"
            ),
            QuizQuestion(
                id: "math-gp-03",
                prompt: "三角形の2つの内角が50°と60°です。残りの角は？",
                concept: "三角形の内角",
                acceptedAnswers: ["70", "70°", "70度"],
                misconceptionAnswers: ["110": "既知の角だけを加算", "290": "一回転の角度から引いている"],
                genericMisconception: "三角形の内角の和の理解不足",
                hints: ["三角形の内角の和を思い出そう。", "3つの角を足すと180°です。", "180−(50+60)を計算しよう。"],
                explanation: "三角形の内角の和は180°なので、180−110=70°です。"
            ),
            QuizQuestion(
                id: "math-gp-04",
                prompt: "半径3cmの円の面積を、πを使って表すと？",
                concept: "円の面積",
                acceptedAnswers: ["9π", "9πcm2", "9πcm²", "9π平方cm"],
                misconceptionAnswers: ["6π": "面積と円周の公式を混同", "3π": "半径を2乗していない"],
                genericMisconception: "円の面積公式の理解不足",
                hints: ["円の面積は半径×半径×πです。", "半径3を2回かけます。", "3×3×πを計算しよう。"],
                explanation: "3×3×π=9πcm²です。"
            )
        ]
    )

    static let chemicalChanges = Quiz(
        id: "chemical-changes-01",
        title: "化学変化 ミニチェック",
        subject: "中学理科",
        topic: "化学変化と物質",
        questions: [
            QuizQuestion(
                id: "science-chem-01",
                prompt: "物質が酸素と結びつく化学変化を何という？",
                concept: "酸化",
                acceptedAnswers: ["酸化"],
                misconceptionAnswers: ["還元": "酸化と還元の混同", "燃焼": "酸化と激しい酸化の混同"],
                genericMisconception: "酸化の定義の理解不足",
                hints: ["変化の前後で酸素が増えています。", "鉄がさびる変化もこの一種です。", "「酸素と化合する」の最初の漢字を手がかりにしよう。"],
                explanation: "物質が酸素と結びつく変化を酸化といいます。"
            ),
            QuizQuestion(
                id: "science-chem-02",
                prompt: "酸化銅から酸素を取り除く化学変化を何という？",
                concept: "還元",
                acceptedAnswers: ["還元"],
                misconceptionAnswers: ["酸化": "酸化と還元の混同", "分解": "化学変化の分類の混同"],
                genericMisconception: "還元の定義の理解不足",
                hints: ["酸素が結びつく変化とは反対です。", "酸化物から酸素が失われます。", "酸化の反対にあたる二字熟語です。"],
                explanation: "酸化物から酸素を取り除く変化を還元といいます。"
            ),
            QuizQuestion(
                id: "science-chem-03",
                prompt: "水を電気分解したとき、水素と酸素の体積比（水素:酸素）は？",
                concept: "水の電気分解",
                acceptedAnswers: ["2:1", "2対1", "2たい1"],
                misconceptionAnswers: ["1:1": "発生する気体の体積が同じだと誤認", "1:2": "水素と酸素の順序が逆"],
                genericMisconception: "水の電気分解の量的関係の理解不足",
                hints: ["水の化学式H₂Oに注目しよう。", "水素は酸素の2倍の体積が発生します。", "水素を先に書く比です。"],
                explanation: "水素は酸素の2倍発生するので、体積比は2:1です。"
            ),
            QuizQuestion(
                id: "science-chem-04",
                prompt: "密閉した容器内の化学変化で、変化の前後に保たれる物質全体の量は？",
                concept: "質量保存の法則",
                acceptedAnswers: ["質量", "重さ"],
                misconceptionAnswers: ["体積": "質量と体積の混同", "密度": "質量と密度の混同"],
                genericMisconception: "質量保存の法則の理解不足",
                hints: ["容器の外へ物質は出入りしません。", "物質の見た目や体積は変わることがあります。", "「○○保存の法則」の○○を答えよう。"],
                explanation: "閉じた系では、化学変化の前後で物質全体の質量は変わりません。"
            )
        ]
    )

    static let japaneseGrammar = Quiz(
        id: "japanese-grammar-01",
        title: "文法・言葉 ミニチェック",
        subject: "中学国語",
        topic: "文法と接続関係",
        questions: [
            QuizQuestion(
                id: "japanese-01",
                prompt: "「静かな町」の「静かな」の品詞は？",
                concept: "品詞",
                acceptedAnswers: ["形容動詞"],
                misconceptionAnswers: ["形容詞": "形容詞と形容動詞の混同", "連体詞": "活用する語と連体詞の混同"],
                genericMisconception: "品詞の識別不足",
                hints: ["言い切りの形に直してみよう。", "言い切りは「静かだ」です。", "語幹に「だ」を付けて言い切る品詞です。"],
                explanation: "「静かな」は「静かだ」と活用する形容動詞です。"
            ),
            QuizQuestion(
                id: "japanese-02",
                prompt: "「私は図書館で本を読む。」の主語は？",
                concept: "文の成分",
                acceptedAnswers: ["私", "私は"],
                misconceptionAnswers: ["図書館": "場所を表す修飾語との混同", "本": "目的語との混同"],
                genericMisconception: "主語の識別不足",
                hints: ["「誰が・何が」に当たる部分を探そう。", "本を読むのは誰でしょう。", "助詞「は」が付いた文節に注目しよう。"],
                explanation: "「誰が読むのか」を表す「私は」が主語です。"
            ),
            QuizQuestion(
                id: "japanese-03",
                prompt: "「雨が降った。しかし、試合は行われた。」の「しかし」が表す接続関係は？",
                concept: "接続語",
                acceptedAnswers: ["逆接"],
                misconceptionAnswers: ["順接": "順接と逆接の混同", "並列": "並列と逆接の混同"],
                genericMisconception: "接続関係の理解不足",
                hints: ["前後の内容は予想どおりにつながっていますか。", "雨なら中止になりそうですが、結果は反対です。", "予想と逆の結果を結ぶ関係です。"],
                explanation: "前の内容から予想される結果と逆の内容を結ぶので、逆接です。"
            ),
            QuizQuestion(
                id: "japanese-04",
                prompt: "「まるで宝石のように輝く」で使われている表現技法は？",
                concept: "表現技法",
                acceptedAnswers: ["直喩", "比喩", "明喩"],
                misconceptionAnswers: ["隠喩": "直喩と隠喩の混同", "擬人法": "比喩表現の種類の混同"],
                genericMisconception: "比喩表現の識別不足",
                hints: ["「ように」という言葉に注目しよう。", "別のものに、目印となる言葉を使ってたとえています。", "「ようだ」「まるで」を使う比喩です。"],
                explanation: "「まるで」「ように」を使って明示的にたとえる直喩です。"
            )
        ]
    )

    static let englishGrammar = Quiz(
        id: "english-grammar-01",
        title: "英文法 ミニチェック",
        subject: "中学英語",
        topic: "時制・比較・語順",
        questions: [
            QuizQuestion(
                id: "english-01",
                prompt: "I ( go ) to the park yesterday. goを適切な形にすると？",
                concept: "過去形",
                acceptedAnswers: ["went"],
                misconceptionAnswers: ["goed": "不規則動詞を規則変化させている", "go": "時制の読み落とし"],
                genericMisconception: "過去形の理解不足",
                hints: ["yesterdayは過去を表します。", "goは不規則動詞です。", "go–went–goneの2番目を使おう。"],
                explanation: "yesterdayがあるため過去形を使い、goはwentになります。"
            ),
            QuizQuestion(
                id: "english-02",
                prompt: "She ___ tennis every Sunday. (play)",
                concept: "三人称単数現在",
                acceptedAnswers: ["plays"],
                misconceptionAnswers: ["play": "三単現のsの付け忘れ", "played": "習慣と過去の混同"],
                genericMisconception: "三人称単数現在の理解不足",
                hints: ["every Sundayは習慣を表します。", "主語Sheは三人称単数です。", "現在形の動詞playの末尾にsを付けよう。"],
                explanation: "現在の習慣で主語がSheなので、動詞はplaysです。"
            ),
            QuizQuestion(
                id: "english-03",
                prompt: "This book is ___ than that one. (interesting)",
                concept: "比較級",
                acceptedAnswers: ["more interesting", "moreinteresting"],
                misconceptionAnswers: ["interestinger": "長い形容詞に-erを付けている", "most interesting": "比較級と最上級の混同"],
                genericMisconception: "比較級の作り方の理解不足",
                hints: ["thanは2つを比べる合図です。", "interestingは比較的長い形容詞です。", "形容詞の前にmoreを置こう。"],
                explanation: "長い形容詞interestingの比較級はmore interestingです。"
            ),
            QuizQuestion(
                id: "english-04",
                prompt: "「彼は今、宿題をしています」を英語にすると？",
                concept: "現在進行形",
                acceptedAnswers: ["he is doing his homework now", "he's doing his homework now", "he is doing homework now", "he's doing homework now"],
                misconceptionAnswers: ["he does his homework now": "現在形と現在進行形の混同", "he doing his homework now": "be動詞の欠落"],
                genericMisconception: "現在進行形の語順の理解不足",
                hints: ["「今〜している」は現在進行形です。", "現在進行形はbe動詞＋動詞ingです。", "主語Heに合うbe動詞isを使おう。"],
                explanation: "現在進行形の形にして、He is doing his homework now.となります。"
            )
        ]
    )

    static let japaneseGeographyHistory = Quiz(
        id: "social-geography-history-01",
        title: "地理・歴史 ミニチェック",
        subject: "中学社会",
        topic: "日本の地理と近世・近代",
        questions: [
            QuizQuestion(
                id: "social-01",
                prompt: "日本で最も面積が大きい都道府県は？",
                concept: "日本の都道府県",
                acceptedAnswers: ["北海道"],
                misconceptionAnswers: ["岩手県": "都道府県の面積順位の混同", "長野県": "都道府県の面積順位の混同"],
                genericMisconception: "都道府県の地理知識不足",
                hints: ["日本列島の北にあります。", "本州ではない大きな島です。", "道庁所在地は札幌市です。"],
                explanation: "日本で最も面積が大きい都道府県は北海道です。"
            ),
            QuizQuestion(
                id: "social-02",
                prompt: "日本標準時の基準となる東経135度の子午線が通る兵庫県の市は？",
                concept: "標準時と経度",
                acceptedAnswers: ["明石市", "明石"],
                misconceptionAnswers: ["神戸市": "兵庫県の県庁所在地との混同", "東京": "首都と標準時子午線の混同"],
                genericMisconception: "標準時子午線の地理知識不足",
                hints: ["兵庫県南部の市です。", "子午線上に天文科学館があります。", "「日本標準時のまち」と呼ばれます。"],
                explanation: "東経135度の日本標準時子午線は兵庫県明石市を通ります。"
            ),
            QuizQuestion(
                id: "social-03",
                prompt: "徳川家康が江戸幕府を開いた年は？",
                concept: "江戸幕府の成立",
                acceptedAnswers: ["1603", "1603年"],
                misconceptionAnswers: ["1600": "関ヶ原の戦いとの混同", "鎌倉時代": "時代区分の混同"],
                genericMisconception: "江戸幕府成立年の理解不足",
                hints: ["関ヶ原の戦いの3年後です。", "17世紀の初めです。", "西暦1600に3を足そう。"],
                explanation: "徳川家康は1603年に征夷大将軍となり、江戸幕府を開きました。"
            ),
            QuizQuestion(
                id: "social-04",
                prompt: "明治政府が藩を廃止して府県を置いた政策は？",
                concept: "明治維新",
                acceptedAnswers: ["廃藩置県"],
                misconceptionAnswers: ["版籍奉還": "版籍奉還と廃藩置県の混同", "大政奉還": "江戸幕府の終結過程との混同"],
                genericMisconception: "明治初期の政策の理解不足",
                hints: ["中央集権化を進めた政策です。", "藩を「廃」し、府県を「置」きました。", "問題文の言葉を四字熟語にまとめよう。"],
                explanation: "藩を廃止し府県を置いた政策を廃藩置県といいます。"
            )
        ]
    )

    static let quizzes: [Quiz] = [
        linearFunctions,
        geometryAndProbability,
        chemicalChanges,
        japaneseGrammar,
        englishGrammar,
        japaneseGeographyHistory
    ]

    static var subjects: [String] {
        Array(Set(quizzes.map(\.subject))).sorted()
    }

    /// 選択中の教材に対応した教師画面用データを生成する。
    static func demoEvents(for quiz: Quiz) -> [AnalysisEvent] {
        guard !quiz.questions.isEmpty else { return [] }
        return (0..<8).map { index in
            let question = quiz.questions[index % quiz.questions.count]
            let isCorrect = index % 3 == 1
            let knownMisconception = question.misconceptionAnswers.values.sorted().first
            return AnalysisEvent(
                participantToken: "P-\(101 + index)",
                questionID: question.id,
                concept: question.concept,
                misconception: isCorrect ? nil : (knownMisconception ?? question.genericMisconception),
                correct: isCorrect,
                hintCount: isCorrect ? 0 : (index % 3) + 1,
                retrySuccess: !isCorrect && index % 2 == 0
            )
        }
    }
}
