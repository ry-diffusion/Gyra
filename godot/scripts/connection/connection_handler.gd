extends Control

var has_errored = false

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	GyraSingleton.login()

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	if has_errored:
		return
		
	var result = GyraSingleton.poll_login()
	var error = result.get("error")
	if error:
		%ErrorWindow.show()
		%ErrorWindow/Contents/Elements/Details.text = "Can't connect to server: %s" % error
		%ErrorWindow/Contents/Elements/Confirm.pressed.connect(_goBack)
		
		has_errored = true
	
	var status = result.get("data")
	
	if status == "gpoll_ok":
		GyraSingleton.switch_to_play_state()
	
	if status == "can_play":
		get_tree().change_scene_to_file("res://scenes/playing.tscn")
func _goBack():
	GyraSingleton.reset_connection()
	get_tree().change_scene_to_file("res://scenes/start_screen.tscn")
