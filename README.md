# Polymer - A flexible cross platform status bar

Polymer is a fast, performant, cross platform status bar.

Nearly all aspects of the status bar can be configured with Lua.

Currently, drawing of the Cairo graphics layer to the screen must be implemented on
each operating system separately, so far this is only implemented for macOS.

## Note

If you get `attempt to index a nil value (field '_module')` when running the code, this is likely
[pavouk/lgi#151](https://github.com/pavouk/lgi/issues/151).  A patch for this issue is just to run
this command.

`ln -s /usr/local/Cellar/cairo/*/lib/libcairo.2.dylib /usr/local/Cellar/gobject-introspection/*/lib/`