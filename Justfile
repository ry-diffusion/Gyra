project-path := "godot"

editor := if os() == 'windows' {
    "bin/Godot_v4.3-stable_win64.exe"
} else {
    "bin/Godot_v4.3-stable_linux64"
}


editor:
    {{editor}} -e --path {{project-path}}