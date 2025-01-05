extends MarginContainer

@onready var current_gpu = $CurrentGPU

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var server = RenderingServer.get_rendering_device()
	current_gpu.text = "GPU: " + server.get_device_name()

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
	pass
