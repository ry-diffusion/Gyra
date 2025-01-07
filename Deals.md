## Chunk System
- [ ] Greedy Meshing
    - [ ] Greedy Meshing with SIMD (SSE/AVX) 
    - [ ] Greedy Meshing with Compute Shaders (This gonna be cool)
- [ ] Dual Contouring?
- [ ] Marching Cubes?
- [ ] Chunk Caching


### Why chunk caching with LOD?
This is help the performance with survival maps, generally the player will not update the entire map, so we can cache the chunks that are not being updated and load them when the player is near and by creating a LOD system we can reduce the number of vertices and faces of the chunks that are far from the player.


## Rendering
- [ ] Dynamic LOD (Level of Detail)
- [ ] Virtual Geometry
- [ ] Ambient Occlusion
- [ ] Global Illumination?

## Extras
- [ ] Modding Support
    - [ ] Swift as main language (swift-bridge, SwiftGodot)
    - [ ] Wasm support? (Secure but I don't know if it will be fast enough, but universal language support!!!)
    - [ ] Native .so/dll support? (It will be insecure, but blazing fast 🚀)

