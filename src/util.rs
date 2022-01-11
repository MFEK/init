use std::path;

pub static METAINFO_PLIST: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>creator</key>
    <string>org.MFEK</string>
    <key>formatVersion</key>
    <integer>3</integer>
</dict>
</plist>"#;

pub static LAYERCONTENTS_PLIST: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<array>
	<array>
		<string>public.default</string>
		<string>glyphs</string>
	</array>
</array>
</plist>"#;

pub static CONTENTS_PLIST: &[u8] = br#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
</dict>
</plist>"#;

pub static TOPLEVEL_WRITTEN: &[(&[u8], &[u8])] = &[(b"metainfo.plist", METAINFO_PLIST), (b"layercontents.plist", LAYERCONTENTS_PLIST)];
pub static GLYPHSDIR_WRITTEN: &[(&[u8], &[u8])] = &[(b"contents.plist", CONTENTS_PLIST)];
pub type FilesToWrite = Vec<(bool, path::PathBuf, &'static [u8])>;
