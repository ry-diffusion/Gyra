extends VBoxContainer

@onready var status: RichTextLabel = $Status
@onready var textBuffer = ""
var threads = []

func _ready() -> void:
	$Server.text_changed.connect(_onServerTextChanged)
	$Play.pressed.connect(_onPlayPressed)
	var t = Thread.new()
	t.start(_checkServer.bind($Server.text))
	threads.push_front(t)

	
func _onPlayPressed() -> void:
	var username = $Username.text
	var server = $Server.text

func _onServerTextChanged(text: String) -> void:
	if text == "":
		$Status.text = "[center]Please Insert a valid address.[/center]"
		return

	var t = Thread.new()
	t.start(_checkServer.bind(text))
	threads.push_front(t)

func _colorize_ping(latency: int) -> String:
	if latency < 100:
		return "[color=green]" + str(latency) + "ms[/color]"
	elif latency < 200:
		return "[color=yellow]" + str(latency) + "ms[/color]"
	else:
		return "[color=red]" + str(latency) + "ms[/color]"

func _checkServer(server: String) -> void:
	var result = NetUtils.query_server_info(server)

	if result.get("error"):
		textBuffer = "[center][color=red]Error: " + result.get("error") + "[/color][/center]"
	else:
		var data = result.get("data")
		print(data)

		var latency = data.get("latency")
		var info = JSON.parse_string(data.get("information"))
		var players = info.get("players")
		var description = info.get("description")
		var description_text = "unsupported description"

		if description.has("text"):
			description_text = description.get("text")

		textBuffer = "[center]%s\n" % description_text
		textBuffer += "Players: %s/%s\n" % [players.get("online"), players.get("max")]
		textBuffer += "Latency: %s[/center]" % _colorize_ping(latency)


	call_deferred("_updateRichText")

func _updateRichText() -> void:
	status.text = textBuffer

func _process(_delta: float) -> void:
	if len(threads) < 0:
		return
	
	for i in range(len(threads)):
		var thread: Thread = threads[i]

		if not thread.is_alive():
			thread.wait_to_finish()
			threads.remove_at(i)
			break
