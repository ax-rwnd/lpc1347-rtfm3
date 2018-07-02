target extended-remote :3333

monitor arm semihosting enable
monitor adapter_khz 2000

define reload
  echo Reloading...
  load
  monitor reset init
  c
end
