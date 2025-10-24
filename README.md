# Planetary Terrain Renderer

![Screenshot 2025-04-26 at 15 13 40](https://github.com/user-attachments/assets/b3101705-20fc-4b6a-abfd-98e2a120b6f3)

A large-scale planetary terrain renderer written in Rust using the Bevy game engine.

This project is developed by [Kurt Kühnert](https://github.com/kurtkuehnert) and contains the reference implementation of my master thesis.
This terrain renderer focuses on visualizing large-scale terrains in a seamless, continuous, and efficient manner.
The source code was developed as the open-source plugin **[bevy_terrain](https://github.com/kurtkuehnert/bevy_terrain)** for the Bevy game engine.

My Master Thesis is as of now not publically released, but will be available soon.

This [Video](https://youtu.be/zdz3-77-3EI) showcases the capabilities and features of this terrain renderer.

## Abstract

Realtime rendering of virtual globes represents the pinnacle of largescale terrain
rendering. Modeling the surface of an entire planet has vast applications, ranging
from Geographic Information Systems (GIS) to educational software. However, the
immense scale of planetary terrain introduces significant challenges, including
levelofdetail (LOD) management and numerical precision limitations. This thesis
provides an overview of the fundamental challenges in planetary terrain rendering
and examines existing solutions. Building on this foundation, a comprehensive
framework for planetary terrain rendering is presented, supporting terrains on an
ellipsoidal base shape, which accurately represents the true spheroid form of planets,
such as the WGS84 reference ellipsoid. The framework covers key aspects such
as viewdependent terrain geometry management, terrain data streaming, and an
accurate spatial reference system (SRS) that integrates seamlessly with the quadtree
based subdivision of terrain geometry and data. A novel approach to maintaining high
precision despite the limitations of floatingpoint accuracy on the Graphics Process
ing Unit (GPU) is introduced. This method leverages a Taylor series approximation
to compute positions on the ellipsoidal surface relative to the viewer. Additionally, a
hierarchical system of coordinate transformations is proposed to accurately represent
terrain positions at various scales. A crucial feature of any virtual globe framework is
its ability to render multiple localized datasets on top of the planetary surface. This
thesis presents a method for achieving this, supported by a preprocessing pipeline
that converts arbitrary georeferenced raster files into datasets compatible with the
rendering system. An extensive opensource reference implementation is provided,
and the framework is evaluated using multiple datasets.

## Screenshots

![10](https://github.com/user-attachments/assets/da19c3b7-dad4-40f1-a94c-f4d987017ca2)
![Screenshot 2025-04-26 at 15 19 18](https://github.com/user-attachments/assets/5b92bb08-ecce-4194-beca-ff7a5b35ade4)
![11](https://github.com/user-attachments/assets/cc82078b-677c-4c2a-8ddf-5f1d2f444882)

## Examples

To try out the terrain renderer, you first have to preprocess your dataset (GeoTIFF).
Some example datasets are available [here](https://drive.proton.me/urls/ZRDAC9SWTM#IxwKkKWSBgnV).
Use the preprocess CLI or a prepared configuration in the `preprocess/examples` directory.
Then run the `examples/spherical.rs` demo with the preprocessed dataset selected.
The default path for the datasets is `source_data`.

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

Planetary Terrain Renderer source code is dual-licensed under either:

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

The Thesis.pdf is excluded from both of these and is licensed under
the [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) license instead.
