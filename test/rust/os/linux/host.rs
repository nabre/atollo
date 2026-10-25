use super::*;

#[test]
fn appimage_only_from_inside_its_mount() {
    let os = |s: &str| Some(std::ffi::OsString::from(s));
    let exe = Path::new("/tmp/.mount_ab/usr/bin/Atollo");
    let (image, dir) = (os("/media/u/K/Atollo.AppImage"), os("/tmp/.mount_ab"));
    assert_eq!(
        appimage_of(exe, image.as_deref(), dir.as_deref()),
        Some(PathBuf::from("/media/u/K/Atollo.AppImage"))
    );
    // A plain binary started from inside another AppImage: not its image.
    let plain = Path::new("/media/u/K/Atollo");
    assert_eq!(appimage_of(plain, image.as_deref(), dir.as_deref()), None);
    assert_eq!(appimage_of(exe, image.as_deref(), None), None);
    assert_eq!(appimage_of(exe, os("").as_deref(), dir.as_deref()), None);
}
