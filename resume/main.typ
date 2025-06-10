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
                [#align(center,"りれきしょ")],
                [#align(start,"たろう")]
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
                    #pad(y: 0.4cm,align(center + horizon,text(nameFontSize,"履歴書")))
                  ],
                  [
                    #pad(y: 0.4cm,align(start + horizon,text(nameFontSize,"太郎")))
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
                  pad(y: 0.2cm,[#addSpace(text(inputFontSize,["平成xx年xx月xx日" 生 #h(0.6cm) (満 #h(0.5em) 99 才)]))])
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
                #image("image/testImage.png", width: 3cm, height: 4cm)
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
                  [#align(center,"とうきょうとすみだくおしあげ")]
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
            電話 #h(10pt) 123-4567-8901
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
              [現住所 (〒 #text(tracking: 1pt,systemFontSize,"131-0045"))]
              #pad(y: 0.2cm ,align(center,text(inputFontSize,"東京都墨田区押上１丁目１−２")))
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
              #pad(y: 0.3cm ,align(center,"sample@example.com"))
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
                  [#align(center,"")]
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
            電話 #h(10pt)
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
              #pad(y: 0.2cm ,align(center,text(inputFontSize,"https://github.com/Nikudanngo/typst-ja-resume-template")))
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
              #pad(y: 0.3cm ,align(center,""))
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
                align(center,[学歴・職歴(各別にまとめて書く)])
              )
            ]
          )
          #place(
            start + top,
            dy: 10pt,
            [
              #let n = 0
              #while n < 14 {
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
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"")
                ],
                [
                  #align(center,"")
                ],
                [
                  #align(center,[学歴])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"平成1")
                ],
                [
                  #align(center,"10")
                ],
                [
                  #align(start + horizon,[#h(5pt)"俺、爆誕"])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"平成20")
                ],
                [
                  #align(center,"3")
                ],
                [
                  #align(start + horizon,[#h(5pt)"スクスク育つ"])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"平成30")
                ],
                [
                  #align(center,"4")
                ],
                [
                  #align(start + horizon,[#h(5pt)"宇宙大学ツヨツヨ学部エンジニア学科 入学"])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"令和1")
                ],
                [
                  #align(center,"8")
                ],
                [
                  #align(start + horizon,[#h(5pt)"大規模開発サークル設立 \u{2192} サークル崩壊"])
                ]
              )
              #linebreak()
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"")
                ],
                [
                  #align(center,"")
                ],
                [
                  #align(center,[職歴])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [
                  #align(center,"令和6")
                ],
                [
                  #align(center,"4")
                ],
                [
                  #align(start + horizon,[#h(5pt)"大手IT系メーカーベンチャー企業 就職"])
                ]
              )
              #grid(
                columns: (1.5cm,0.8cm,1fr),
                [],
                [],
                [
                  #align(end + horizon,[以上#h(2cm)])
                ]
              )
            ]
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
              align(center,[学歴・職歴(各別にまとめて書く)])
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
      ]
    ),
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
      height: 6.6cm,
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
            #while n < 7 {
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
            #grid(
              columns: (1.5cm,0.8cm,1fr),
              [
                #align(center,"平成1")
              ],
              [
                #align(center,"12")
              ],
              [
                #align(start + horizon,[#h(5pt)"普通自動車免許 取得"])
              ]
            )
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
    [私がこの職に応募する理由は、]
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
    [私は〇〇がしたい]
      ]
    )
  ),
  place(
    bottom + right,
    dy: 10pt,
    [Made with Typst]
  )
)
