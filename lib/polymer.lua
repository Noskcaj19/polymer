local polymer = {}

-- Defined in rust
polymer.connect_signal = __polymer_sys.connect_signal
polymer.emit_signal = __polymer_sys.emit_signal

polymer.info = __polymer_sys.info
polymer.warn = __polymer_sys.warn
polymer.error = __polymer_sys.error

return polymer
