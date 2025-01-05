project-path := "godot"
native-src-path := "rust"

editor := if os() == 'windows' {
    "bin/Godot_v4.3-stable_win64.exe"
} else {
    "bin/Godot_v4.3-stable_linux64"
}


editor:
    {{editor}} -e --path {{project-path}}

watch:
    cargo watch -C {{native-src-path}} -x build