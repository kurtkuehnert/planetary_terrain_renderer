# Bevy Terrain

![GitHub](https://img.shields.io/github/license/Ku95/bevy_terrain)
![Discord](https://img.shields.io/discord/999221999517843456?label=discord)
<!--
![Crates.io](https://img.shields.io/crates/v/bevy_terrain)
![docs.rs](https://img.shields.io/docsrs/bevy_terrain)
-->

Bevy Terrain is a plugin for rendering terrains with the Bevy game engine.
This plugin supports planar[^1] and spherical/ellipsoidal terrains of (almost) any size and resolution.

![](https://user-images.githubusercontent.com/51823519/202845032-0537e929-b13c-410b-8072-4c5b5df9830d.png)
(Data Source: Federal Office of Topography, [©swisstopo](https://www.swisstopo.admin.ch/en/home.html))

**Warning:** This plugin is still in early development, so expect the API to change and possibly break your existing
code.

Bevy Terrain was developed as part of my [bachelor thesis](https://github.com/kurtkuehnert/terrain_renderer) and
my [master thesis](https://github.com/kurtkuehnert/spherical_terrain_renderer) on the
topic of large-scale terrain rendering.
If you would like to help me build an extensive open-source terrain rendering library for the Bevy game engine, feel
free to contribute to the project.
Also, join the Bevy Terrain [Discord server](https://discord.gg/7mtZWEpA82) for help, feedback, or to discuss feature
ideas.

## Examples

To try out Bevy Terrain, you first have to preprocess your dataset (GeoTIFF).
Some example datasets are available [here](https://drive.proton.me/urls/ZRDAC9SWTM#IxwKkKWSBgnV).
Use the preprocess CLI or a prepared configuration in the `preprocess/examples` directory.
Then run the `examples/spherical.rs` demo with the preprocessed dataset selected.
The default path for the datasets is `assets/source_data`.

## Documentation

The `docs` folder contains a
high-level [implementation overview](https://github.com/kurtkuehnert/bevy_terrain/blob/main/docs/implementation.md),
as well as the [development status](https://github.com/kurtkuehnert/bevy_terrain/blob/main/docs/development.md),
enumerating the features that I am planning on implementing next.
If you would like to contribute to the project, this is a good place to start. Simply pick an issue/feature and discuss
the details with me on Discord or GitHub.
I would also recommend you take a look at
my [thesis](https://github.com/kurtkuehnert/terrain_renderer/blob/main/Thesis.pdf).
There, I present the basics of terrain rendering (chapter 2), common approaches (chapter 3), and a detailed explanation
of the
method used by `bevy_terrain` (chapter 4).

## Debug Controls

These are the debug controls of the plugin.
Use them to navigate the terrain, experiment with the quality settings, and enter the different debug views.
There are two camera controller options available: a fly camera for navigating using the keyboard and an orbital camera
using only the mouse.

### Fly Camera

- `T` - toggle fly camera movement
- Move the mouse to look around
- Press the arrow keys to move the camera horizontally
- Use `PageUp` and `PageDown` to move the camera vertically
- Use `Home` and `End` to increase/decrease the camera's movement speed

### Orbital Camera

- `R` - toggle orbital camera movement
- Hold the left mouse button to pan the camera
- Hold the middle mouse button to rotate the camera
- Hold the right mouse button to zoom the camera

### Visualization Toggles

- `W` - toggle wireframe view
- `L` - toggle terrain data LOD view
- `Y` - toggle terrain geometry LOD view
- `Q` - toggle tile tree view
- `P` - toggle pixel view
- `U` - toggle UV view
- `B` - toggle normals view
- `M` - toggle morphing
- `K` - toggle blending
- `Z` - toggle tile tree LOD
- `S` - toggle lighting
- `G` - toggle texture sampling using gradients
- `H` - toggle high precision coordinates
- `F` - toggle freeze view frustum
- `D` - toggle surface approximation debug

### Quality Adjustments

- `N` - decrease blend distance
- `E` - increase blend distance
- `I` - decrease morph distance
- `O` - increase morph distance
- `X` - decrease grid size
- `J` - increase grid size

## GPU Frame Capture (macOS)

When enabling the `metal_capture` feature, you can trigger a GPU frame capture using the `C` key.
Recorded captures are stored in the `captures` directory of the project.
They can be examined and analyzed using Xcode.

## Attribution

The examples use the following [demo datasets](https://drive.proton.me/urls/ZRDAC9SWTM#IxwKkKWSBgnV):

- GEBCO Compilation Group (2023) - GEBCO 2023 Grid
- Unearthed Outdoors - True Marble Global Image Dataset GeoTIFF - [Creative Commons Attribution 3.0 United States
  License](https://creativecommons.org/licenses/by/3.0/us/legalcode)
- ©swisstopo - swissALTIRegio
- This work utilizes data made available under the Norwegian Licence for Open Government Data (NLOD), distributed by the
  Norwegian Offshore Directorate. The data were originally acquired by various entities. For more information on the
  data,
  please visit the Norwegian Offshore Directorate's open data page:
  https://www.sodir.no/en/about-us/open-data/.

## License

Bevy Terrain source code (this excludes the datasets in the assets directory) is dual-licensed under either:

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

## Contributions

Unless explicitly stated otherwise, any contribution intentionally submitted for inclusion in the work, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[^1]: Currently, the support for planar terrain rendering is broken.