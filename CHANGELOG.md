# Changelog

## [0.4.1](https://github.com/tintenfisch-app/Tintenfisch/compare/v0.4.0...v0.4.1) (2026-11-01)


### Bug Fixes

* **annotations:** drop deleted comments with their popups and replies from saved files ([5cb856f](https://github.com/tintenfisch-app/Tintenfisch/commit/5cb856fc1606dadcea920277e13cb1a32edf53f3)), closes [#34](https://github.com/tintenfisch-app/Tintenfisch/issues/34)
* **bookmarks:** cap the indent of deeply nested bookmarks and show full titles on hover ([668e354](https://github.com/tintenfisch-app/Tintenfisch/commit/668e3549a6a4a8783066cfe4902f0abb29e33686)), closes [#30](https://github.com/tintenfisch-app/Tintenfisch/issues/30)
* **comments:** number and order comments by the current page order ([0fddb73](https://github.com/tintenfisch-app/Tintenfisch/commit/0fddb736c9f337ddaaa909be9fc86550ecff914f)), closes [#27](https://github.com/tintenfisch-app/Tintenfisch/issues/27)
* **comments:** type into a new sticky note or text box right after placing it ([86bafb0](https://github.com/tintenfisch-app/Tintenfisch/commit/86bafb0732ee948c54e2d5ceefaf8f0d0ee88a9e)), closes [#26](https://github.com/tintenfisch-app/Tintenfisch/issues/26)
* **edit:** finish a text box with Esc while focus is in the format bar ([0756c56](https://github.com/tintenfisch-app/Tintenfisch/commit/0756c56d4e7dd902b4c4229235d7a094cc2b2212)), closes [#16](https://github.com/tintenfisch-app/Tintenfisch/issues/16)
* **edit:** keep keys typed before a text box or line has opened ([64f450f](https://github.com/tintenfisch-app/Tintenfisch/commit/64f450f88333f57361e3b3b780508b143e572814)), closes [#12](https://github.com/tintenfisch-app/Tintenfisch/issues/12)
* **edit:** keep text edits redoable after undo and bind Ctrl+Y to redo ([fb3288b](https://github.com/tintenfisch-app/Tintenfisch/commit/fb3288b3ae3e05f4e81a8546715f2e5d770ff687)), closes [#13](https://github.com/tintenfisch-app/Tintenfisch/issues/13)
* **edit:** keep the edit panel open when picking a redaction tool from it ([41b8f49](https://github.com/tintenfisch-app/Tintenfisch/commit/41b8f49bc77db79f7edad2ace83ea4c3c8cc019d)), closes [#18](https://github.com/tintenfisch-app/Tintenfisch/issues/18)
* **fonts:** render and extract text with predefined CJK CMaps ([737a788](https://github.com/tintenfisch-app/Tintenfisch/commit/737a788ca5c93806259cc2b54815a874d1fd0cee)), closes [#25](https://github.com/tintenfisch-app/Tintenfisch/issues/25)
* **forms:** enforce MaxLen on text fields ([9594dea](https://github.com/tintenfisch-app/Tintenfisch/commit/9594deaccfb093fd36ddaed36bb5adb0787a3d49)), closes [#22](https://github.com/tintenfisch-app/Tintenfisch/issues/22)
* **forms:** warn when a form carries XFA content ([eb69f87](https://github.com/tintenfisch-app/Tintenfisch/commit/eb69f87194e265963ed97e48353d89b259bd053c)), closes [#23](https://github.com/tintenfisch-app/Tintenfisch/issues/23)
* **images:** downscale and compress large placed pictures ([f33ce0f](https://github.com/tintenfisch-app/Tintenfisch/commit/f33ce0f9ccbc4bab9a0a7fda17f892ab077a4e59)), closes [#21](https://github.com/tintenfisch-app/Tintenfisch/issues/21)
* **library:** reject non-PDF files before storing them ([b2a7950](https://github.com/tintenfisch-app/Tintenfisch/commit/b2a7950a7f9f7315970494745843019c751f24e8)), closes [#31](https://github.com/tintenfisch-app/Tintenfisch/issues/31)
* **library:** warn about other tabs and never overwrite their newer saves ([984c660](https://github.com/tintenfisch-app/Tintenfisch/commit/984c660fccdedc65388e084440d923cc25b8b8f9)), closes [#17](https://github.com/tintenfisch-app/Tintenfisch/issues/17)
* **pages:** write a loop-free page tree when the source tree lists itself ([48f84ff](https://github.com/tintenfisch-app/Tintenfisch/commit/48f84ff7b63e8cb2a5cbb6dfbb04ca6ce4609a6a)), closes [#28](https://github.com/tintenfisch-app/Tintenfisch/issues/28)
* **protect:** enforce owner password permission flags ([26ecf6d](https://github.com/tintenfisch-app/Tintenfisch/commit/26ecf6d0362efa10b69a4c09e10cfba04f58ba55)), closes [#32](https://github.com/tintenfisch-app/Tintenfisch/issues/32)
* **redact:** keep the redaction bar off the page while other tools place content ([9dd3170](https://github.com/tintenfisch-app/Tintenfisch/commit/9dd3170061ee4a051cdf0b4f3b632d8dd75ed570)), closes [#19](https://github.com/tintenfisch-app/Tintenfisch/issues/19)
* **render:** fit substitute glyphs to the declared widths of missing fonts ([2f90fa7](https://github.com/tintenfisch-app/Tintenfisch/commit/2f90fa71d71558f81e141b250f83a44c0243125b)), closes [#29](https://github.com/tintenfisch-app/Tintenfisch/issues/29)
* resolve all open bug reports ([dca6e2b](https://github.com/tintenfisch-app/Tintenfisch/commit/dca6e2baf34fbe6191e668e85900eee036d0ebcf))
* **save:** declare PDF 2.0 when encrypting with AES-256 ([0ff0822](https://github.com/tintenfisch-app/Tintenfisch/commit/0ff0822263c8478cd0005bbfc04a7f4551bc92c4)), closes [#20](https://github.com/tintenfisch-app/Tintenfisch/issues/20)
* **shell:** show the current page count in the document status ([a485968](https://github.com/tintenfisch-app/Tintenfisch/commit/a4859688a991e8c3b0c22b10b330bb68e46fe383)), closes [#14](https://github.com/tintenfisch-app/Tintenfisch/issues/14)
* **signing:** report expired and unverified signer certificates separately ([067d113](https://github.com/tintenfisch-app/Tintenfisch/commit/067d1130a3c90fd87c0ff56197919f460f7c48cb)), closes [#33](https://github.com/tintenfisch-app/Tintenfisch/issues/33)
* **signing:** warn about expired certificates and explain invalid .p12 files ([7544bf9](https://github.com/tintenfisch-app/Tintenfisch/commit/7544bf9fac9b7a1c836c29722bd6bb0eb884e45f)), closes [#24](https://github.com/tintenfisch-app/Tintenfisch/issues/24)
* **tools:** leave the active tool with Esc when nothing is selected ([acc98e9](https://github.com/tintenfisch-app/Tintenfisch/commit/acc98e97cced76235f624412f32649c4a18ec0bb)), closes [#15](https://github.com/tintenfisch-app/Tintenfisch/issues/15)

## [0.4.0](https://github.com/ArifKobel/Tintenfisch/compare/v0.3.0...v0.4.0) (2026-10-22)


### Features

* **desktop:** make the app icon red so it reads as a PDF ([9f7556d](https://github.com/ArifKobel/Tintenfisch/commit/9f7556dd608b1ebbb67f3fad990723153b5880b4))

## [0.3.0](https://github.com/ArifKobel/Tintenfisch/compare/v0.2.0...v0.3.0) (2026-10-22)


### Features

* **desktop:** use a PDF page with the octopus as the app icon ([002a261](https://github.com/ArifKobel/Tintenfisch/commit/002a2618961cda392fbb026d1da36d41d1045929))

## [0.2.0](https://github.com/ArifKobel/Tintenfisch/compare/v0.1.0...v0.2.0) (2026-10-22)


### Features

* **desktop:** add a desktop app that opens PDFs from the system ([23953f3](https://github.com/ArifKobel/Tintenfisch/commit/23953f345f8fa1336badd14fa6661279e6392c1e))
* **desktop:** hide the GitHub star link in the desktop app ([ff7034f](https://github.com/ArifKobel/Tintenfisch/commit/ff7034f80ee2291a15b6bce2224201170a0f17a7))
* **desktop:** save files through a native save dialog ([4f5d4c6](https://github.com/ArifKobel/Tintenfisch/commit/4f5d4c696c5c6065cd94d5bd3dc81a6b900945e9))
* **desktop:** ship Tintenfisch as a macOS and Windows app ([d24daba](https://github.com/ArifKobel/Tintenfisch/commit/d24daba8ae9f7b3e4f40ff702e8afdb33d834ea1))


### Bug Fixes

* **editing:** delete text with Backspace and Delete in WebKit ([cfde0ee](https://github.com/ArifKobel/Tintenfisch/commit/cfde0eeb1d4ee0aadb20c1feb1e9677c8d0fc365))
* **editing:** keep editing when the font picker opens in WebKit ([fa0606b](https://github.com/ArifKobel/Tintenfisch/commit/fa0606b212a21aa44bd19e30110e52fce640cb01))

## 0.1.0 (2026-10-21)

The first release of Tintenfisch, a PDF editor that runs entirely in the browser on its own PDF engine in Rust and WebAssembly.

### Features

* **Edit:** change text in place with reflowing paragraphs and the original fonts where the file allows it, add text and images, move, resize and replace images, add and edit links
* **Comment:** sticky notes, text boxes, highlights, freehand drawing, rectangles, ellipses, arrows and stamps, with a comments panel
* **Organize:** rotate, delete, reorder and extract pages, insert blank pages or pages from another PDF, edit bookmarks
* **Forms:** fill in text fields, checkboxes, radio buttons and lists, create new fields and change their properties
* **Redact:** remove text and areas from the file, not just cover them
* **Sign:** place a drawn or uploaded signature, sign digitally with a PKCS#12 certificate and check existing signatures
* **Protect:** AES-256 password protection with permissions for printing, copying and editing
* **Workspace:** find text, zoom, print, download, recent files with automatic saving in the browser, mobile layout with a bottom toolbar
