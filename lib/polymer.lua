local polymer = {}

-- Defined in rust
polymer.connect_signal = __polymer_sys.connect_signal
polymer.emit_signal = __polymer_sys.emit_signal

return polymer