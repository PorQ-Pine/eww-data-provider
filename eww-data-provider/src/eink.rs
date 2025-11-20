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
    Normal((BitDepth, Redraw)), //0
    Fast(Dithering), // 1
    // Doesn't work for me
    // Zero, // 8
}

// RenderHints
// Only matters in Normal mode
enum BitDepth {
    Y1(Conversion),
    Y2(Conversion),
    Y4,
}

enum Conversion {
    Tresholding, // T
    Dithering, // D
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
