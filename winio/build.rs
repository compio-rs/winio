use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        win32: { all(windows, feature = "win32") },
        winui: { all(windows, feature = "winui") },
    }
}
