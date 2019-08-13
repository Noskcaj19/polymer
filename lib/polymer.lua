local polymer = {}

-- Defined in rust

polymer.request_redraw = __polymer_sys.request_redraw
polymer.connect_signal = __polymer_sys.connect_signal
polymer.emit_signal = __polymer_sys.emit_signal
polymer.add_timer = __polymer_sys.add_timer

polymer.trace = __polymer_sys.trace
polymer.info = __polymer_sys.info
polymer.warn = __polymer_sys.warn
polymer.error = __polymer_sys.error

return polymer
