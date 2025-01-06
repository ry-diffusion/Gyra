extends RichTextLabel


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
	var result = GyraSingleton.query_login_status()
	if result.get("error"):
		# Handle error
		return
		
	var raw_status = result.get("data").get("status")
	
	text = "[center][color=green]%s[/color][/center]" % raw_status
