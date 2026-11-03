# Court dialogue font

`court-dialogue.ttf` contains the complete **Noto Sans CJK SC 2.004** font,
regular weight (400), converted to static TrueType. All 65,535 source glyphs
and 44,810 Unicode character mappings are preserved; the font is not subsetted.

Source: the upstream static `NotoSansCJKsc-Regular.otf` (Simplified Chinese).
[Upstream project](https://github.com/notofonts/noto-cjk).

Converted with FontTools 4.63.0: convert all cubic CFF outlines to quadratic
TrueType outlines with `Cu2QuPen(max_err=1, reverse_direction=True)` and rename
the resulting family to **Court Dialogue**. Character mappings and layout
tables are retained. The TrueType outlines allow loading by Bevy's font
renderer. No font tool is needed at runtime.

The font remains licensed under the SIL Open Font License 1.1; see
[OFL.txt](OFL.txt).
