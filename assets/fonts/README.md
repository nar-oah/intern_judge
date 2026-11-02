`CourtDialogueSans.otf` 是 [Noto Sans CJK SC 2.004 Regular](https://github.com/notofonts/noto-cjk/blob/Sans2.004/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf) 的静态子集，供法院 NPC 和底部对话框使用。保留 ASCII U+0020–U+007E 和当前法院中文台词、按钮、操作提示所需字符，约 33 KB；名称已改为 Court Dialogue Sans。字体沿用 SIL Open Font License 1.1，完整许可见 [OFL.txt](OFL.txt)，原版权和许可也保存在字体元数据中。

扩展台词时应重新生成子集，以免新增中文显示为缺字。下载上述原字体并安装 FontTools 后，可按下面的方式生成（`characters` 应包含 `.rep` 脚本和 UI 中的全部文字）：

```python
from fontTools import subset
from fontTools.ttLib import TTFont

font = TTFont("NotoSansCJKsc-Regular.otf")
options = subset.Options()
options.name_IDs = ["*"]
options.name_legacy = True
options.name_languages = ["*"]
subsetter = subset.Subsetter(options=options)
characters = "公诉方：我们认为被告有罪，请法官依法判决。辩护方：我们认为证据不足，请法官判决无罪。法官请选择本案判决。判决有罪判决无罪按 E 继续靠近按 E 对话。"
subsetter.populate(unicodes=range(0x20, 0x7f), text=characters)
subsetter.subset(font)
names = {1: "Court Dialogue Sans", 2: "Regular", 3: "Court Dialogue Sans Regular 2.004", 4: "Court Dialogue Sans Regular", 6: "CourtDialogueSans-Regular", 16: "Court Dialogue Sans", 17: "Regular"}
for name_id, value in names.items():
    font["name"].removeNames(nameID=name_id)
    font["name"].setName(value, name_id, 3, 1, 0x409)
font.save("CourtDialogueSans.otf")
```
