// Only matters when:
// DriverMode is Fast
// Normal, Y2 and Y1
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DitherMode y 0
enum Dithering {
    Bayer, //0
    BlueNoise16, // 1
    BlueNoise32, // 2
}

// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DriverMode y 0
enum DriverMode {
    Normal(BitDepth), //0
    Fast(Dithering), // 1
    // Doesn't work for me
    // Zero, // 8
}

// RenderHints
// Only matters in Normal mode
// busctl --user set-property org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 DefaultHintHr s "Y1|T|R"
enum BitDepth {
    Y1(Conversion, Redraw),
    Y2(Conversion, Redraw),
    Y4,
}

enum Conversion {
    Tresholding, // T
    Dithering(Dithering), // D
}

enum Redraw {
    FastDrawing, // R
    DisableFastDrawing, // r
}

enum ScreenOptions {
    // busctl --user call org.pinenote.PineNoteCtl /org/pinenote/PineNoteCtl org.pinenote.Ebc1 GlobalRefresh
    FullRefresh,
    ScreenMode(DriverMode),
}
