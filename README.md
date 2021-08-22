# wgpu playground

5 small wgpu demos based on me following the `learn-wgpu` tutorials with basically zero graphics programming experience, glued together into one executable with some particularly overengineered abstractions.

- Textured: `learn-wgpu`'s "Textures and bind groups" tutorial with some changes.
- Cameras: `learn-wgpu`'s "Uniform buffers and a 3d camera", but the shown mesh is tridimensional.
- Instancing: `learn-wgpu`'s "Uniform buffers and a 3d camera" + "The depth buffer", but with ~~128~~ 1089 entities whose model matrices are updated every frame, and with a toggleable depth buffer view.
- "Clown Colors" and "Triangle": fragment shader playaround.

## Controls

`Space` to change the demo being currently displayed.

If available on the current demo, `N` switches the image texture.

On the instancing demo:
- `M` switches the grid animation. Currently supported grid animations are:
    - `DoubleWave` (default): wave animation over a single axis.
    - `Metaball`: metaball animation over a single axis (thanks to @dmitmel for providing implementation pointers).
- `B` toggles from the default view to a grayscale depth buffer view.

## Notes

- Shaders come precompiled as SPIR-V. The GLSL sources for those shaders are available alongside the SPIR-V output. If you are just testing and you not want to waste two decades of your life waiting for this thing to precompile, remove `shaderc` from `Cargo.toml` and move the `build.rs` somewhere else where Cargo can't see it.
