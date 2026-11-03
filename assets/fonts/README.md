# Court dialogue font

`court-dialogue.ttf` is a small, static TrueType subset of **Noto Sans CJK SC
2.004**, regular weight (400). It includes printable ASCII and the characters
used by the court dialogue:

```text
请作出你的判决
← 左方向键：有罪
→ 右方向键：无罪
E 键：离开
公诉方：请求判决有罪。
辩护方：请求判决无罪。
```

Source: `NotoSansCJK-VF.otf.ttc` from the installed `noto-fonts-cjk-sans-2.004`
package, collection face 2 (Simplified Chinese).
[Upstream project](https://github.com/notofonts/noto-cjk).

Generated with FontTools 4.63.0: subset the source to the characters above and
U+0020–U+007E, instantiate the `wght` axis at 400, convert cubic outlines to
quadratic TrueType outlines with `Cu2QuPen(max_err=1, reverse_direction=True)`,
and rename the resulting family to **Court Dialogue**. The TrueType outlines
allow loading by Bevy's font renderer. No font tool is needed at runtime.

The font remains licensed under the SIL Open Font License 1.1; see
[OFL.txt](OFL.txt). Add glyphs to the subset if dialogue text is extended.
