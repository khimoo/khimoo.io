#import "template.typ": *
#let systemFontSize = 8pt
#let nameFontSize = 16pt
#let inputFontSize = 10pt

#let title = [#text(tracking: 1em,size: 14pt,[履歴書])]

#let fontSerif = ("Noto Serif", "Noto Serif CJK JP")
#let fontSan = ("Noto Sans", "Noto Sans CJK JP")

#set text(font: fontSerif, size: systemFontSize)
#set page(paper: "jis-b5",margin: 1.5cm)

#let addSpace(input) = {
  box(
      [#pad(left:1cm,[#input])],
     )
}

#let data = yaml("data.yaml")

= #title
// 使い方の説明。
// "私"と"アドレス"など日本語名の関数の引数を変更してください。

#move( dy: -1cm,
    stack(
      align(bottom,
        grid(
          columns: (5fr,2fr),
          stack(
            place(
              top + right,
              dy: -10pt,
              datetime.today().display(
                "[year]年[month]月[day]日現在",
                )
              ),
            rect(
              stroke: (
                bottom: none,
                top: 1.5pt,
                left: 1.5pt,
                right: 1.5pt
                ),
              height: auto,
              width: 100%,
              [
#grid(
  columns: (1.5cm,4cm,1fr),
  [ふりがな],
  [#align(center,data.sei_yomi)],
  [#align(start,data.mei_yomi)]
  )
              ]
              ),
            line(
                length: 100%,
                stroke: (
                  dash:"dashed",
                  )
                ),
            rect(
                stroke: (
                  bottom: 0.5pt,
                  top: none,
                  left: 1.5pt,
                  right: 1.5pt
                  ),
                height: auto,
                width: 100%,
                [
#align(top,
  grid(
    columns: (1.5cm,4cm,1fr),
    [氏 #h(0.6cm) 名],
    [
#pad(y: 0.4cm,align(center + horizon,text(nameFontSize,data.sei)))
    ],
    [
#pad(y: 0.4cm,align(start + horizon,text(nameFontSize,data.mei)))
    ]
    )
  )
                ]
                ),
                rect(
                    stroke: (
                      bottom: 0.5pt,
                      top: none,
                      left: 1.5pt,
                      right: 1.5pt
                      ),
                    height: auto,
                    width: 100%,
                    [
#align(start + top,
  grid(
    columns: (1.5cm,1fr),
    [生年月日],
    pad(y: 0.2cm,[#addSpace(text(inputFontSize,[#data.birth.wareki 生 #h(0.6cm) (満 #h(0.5em) #data.age 才)]))])
    )
  )
                    ]
                    )
                ),
                [
#set text(size: 7pt)
#pad(
    bottom: 0.3cm,
    left: 0.4cm,
    box(
      stroke: (
        dash:"dashed",
        ),
      height: 4cm,
      width: 3cm,
      [
#image(data.photo, width: 3cm, height: 4cm)
      ]
      )
    )
                ]
                ),
                ),
                stack(
                    grid(
                      columns: (5fr,2fr),
                      [
#stack(
  rect(
    stroke: (
      bottom: none,
      top: none,
      left: 1.5pt,
      right: 0.5pt
      ),[
#grid(
  columns: (1.5cm,1fr),
  [ふりがな],
  [#align(center,data.address1_yomi)]
  )
      ]
    ),
  line(stroke: (dash:"dashed"), length: 100%)
  )
                      ],
                      [
#rect(
    width: 100%,
    stroke: (
      bottom: 0.5pt,
      top: 1.5pt,
      left: none,
      right: 1.5pt
      ),[
    電話 #h(10pt) #data.tel1
      ]
    )
                      ]
                      ),
                      grid(
                          columns: (5fr,2fr),
                          [
#rect(
  width: 100%,
  height: 1.8cm,
  stroke: (
    bottom: 0.5pt,
    top: none,
    left: 1.5pt,
    right: 0.5pt
    ),[
  [現住所 (〒 #text(tracking: 1pt,systemFontSize,data.zip1))]
#pad(y: 0.2cm ,align(center,text(inputFontSize,data.address1)))
    ]
  )
                          ],
                          [
#rect(
  width: 100%,
  height: 1.8cm,
  stroke: (
    bottom: 0.5pt,
    top: none,
    left: none,
    right: 1.5pt
    ),[
  E-mail
#pad(y: 0.3cm ,align(center,data.email1))
    ]
  )
                          ]
                          ),
                          grid(
                              columns: (5fr,2fr),
                              [
#stack(
  rect(
    stroke: (
      bottom: none,
      top: none,
      left: 1.5pt,
      right: 0.5pt
      ),[
#grid(
  columns: (1.5cm,1fr),
  [ふりがな],
  [#align(center,data.address2_yomi)]
  )
      ]
    ),
  line(stroke: (dash:"dashed"), length: 100%)
  )
                              ],
                              [
#rect(
    width: 100%,
    stroke: (
      bottom: 0.5pt,
      top: none,
      left: none,
      right: 1.5pt
      ),[
    電話 #h(10pt) #data.tel2
      ]
    )
                              ]
                              ),
                              grid(
                                  columns: (5fr,2fr),
                                  [
#rect(
  width: 100%,
  height: 1.8cm,
  stroke: (
    bottom: 1.5pt,
    top: none,
    left: 1.5pt,
    right: 0.5pt
    ),[
  [連絡先 (〒 #h(20pt) - #h(20pt))]
#pad(y: 0.2cm ,align(center,text(inputFontSize,data.address2)))
    ]
  )
                                  ],
                                  [
#rect(
  width: 100%,
  height: 1.8cm,
  stroke: (
    bottom: 1.5pt,
    top: none,
    left: none,
    right: 1.5pt
    ),[
  E-mail
#pad(y: 0.3cm ,align(center,data.email2))
    ]
  )
                                  ]
                                  )
                                  ),
                                  linebreak(),
                                  stack(
                                      box(
                                        stroke: (
                                          bottom: 1.5pt,
                                          top: 1.5pt,
                                          left: 1.5pt,
                                          right: 1.5pt
                                          ),
                                        height: 12.6cm,
                                        width: 100%,
                                        [
                                        // You can also import those.
#import table: cell, header

#let box_height = 12.6pt
#let A = (
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center+horizon)[#box(height: box_height)[#text(size:10pt)[学歴]]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  (align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]], align(center + horizon)[#box(height: box_height)[]]),
  )

#let education = data.education
#let experience = data.experience
#let row = 0

#for (i, entry) in education.enumerate() {
  row = row + 1
    let year = entry.at(0)
    let month = entry.at(1)
    let desc = entry.at(2)

    A.at(row) = (
        align(center + horizon)[#box(height: box_height)[#year]],
        align(center + horizon)[#box(height: box_height)[#month]],
        align(left + horizon)[#box(height: box_height)[#desc]]
        )
}

#(A.at(row + 1) = (
      align(center + horizon)[#box(height: box_height)[]],
      align(center + horizon)[#box(height: box_height)[]],
      align(center + horizon)[#box(height: box_height)[#text(size:10pt)[職歴]]]
      ))
#(row = row + 1)

#for (i, entry) in experience.enumerate() {
  row = row + 1
    let year = entry.at(0)
    let month = entry.at(1)
    let desc = entry.at(2)

    A.at(row) = (
        align(center + horizon)[#box(height: box_height)[#year]],
        align(center + horizon)[#box(height: box_height)[#month]],
        align(left + horizon)[#box(height: box_height)[#desc]]
        )
}
#(A.at(row + 1) = (
      align(center + horizon)[#box(height: box_height)[]],
      align(center + horizon)[#box(height: box_height)[]],
      align(right + horizon)[#box(height: box_height, width: 1fr)[#text(size:9pt)[以上]]#box(width:60pt)[]],
      ))

#table(
    columns: (1.5cm, 0.8cm, 1fr),
    stroke: .4pt,
    table.header(
      cell(inset:6pt,[#align(center)[*年*]]),
      cell(inset:6pt,[#align(center)[*月*]]),
      cell(inset:6pt,[#align(center)[*学歴・職歴*]])
      ),
    ..for row in A {(
        ..for val in row {(
            [#val],
            )}
        )}
    )
]
),
  ),
  ),
  )
#pagebreak()

#stack(
    stack(
      box(
        stroke: (
          bottom: 1.5pt,
          top: 1.5pt,
          left: 1.5pt,
          right: 1.5pt
          ),
        height: 5cm,
        width: 100%,
        [
#grid(
  columns: (1.5cm,0.8cm,1fr),
  [
#rect(
  stroke: (
    bottom: none,
    top: none,
    left: none,
    right: 0.5pt
    ),
  height: 100%,
  width: 100%,
  [
#align(center,[年])
  ]
  )
  ],
  [
#rect(
  stroke: (
    bottom: none,
    top: none,
    left: none,
    right: 0.5pt
    ),
  height: 100%,
  width: 100%,
  [
#align(center,[月])
  ]
  )
  ],
  [
#rect(
    width: 100%,
    height: 100%,
    stroke: (
      bottom: none,
      top: none,
      left: none,
      right: none,
      ),
    align(center,[免許・資格])
    )
  ]
  )
#place(
    start + top,
    dy: 10pt,
    [
#let n = 0
#while n < 5 {
    [#pad(y: 0.26cm,line(stroke: 0.5pt, length: 100%))]
    n = n + 1
    }
    ]
    )
#place(
    top + left,
    dy: 0.9cm,
    [
#set text(size: inputFontSize)
#for item in data.licences {
    grid(
        columns: (1.5cm,0.8cm,1fr),
        [
#align(center, [#item.at(0)])
        ],
        [
#align(center, [#item.at(1)])
        ],
        [
#align(start + horizon,[#h(5pt)#item.at(2)])
        ]
        )
    }
    ]
    )
]
),
  ),
  linebreak(),
  stack(
      rect(
        stroke: (
          bottom: 1.5pt,
          top: 1.5pt,
          left: 1.5pt,
          right: 1.5pt
          ),
        height: 5cm,
        width: 100%,
        [
        志望の動機、特技、好きな学科、アピールポイントなど
#linebreak()
#set text(size: inputFontSize)
        [#data.motive]
        ]
        )
      ),
  linebreak(),
  stack(
      rect(
        stroke: (
          bottom: 1.5pt,
          top: 1.5pt,
          left: 1.5pt,
          right: 1.5pt
          ),
        height: 5cm,
        width: 100%,
        [
        本人希望記入欄(特に給料・職種・勤務時間・勤務地・その他についての希望があれば記入)
#linebreak()
#set text(size: inputFontSize)
        [#data.wish]
        ]
        )
      ),
  place(
      bottom + right,
      dy: 10pt,
      [Made with Typst]
      )
  )
