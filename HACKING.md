# Profiling

I have not had much luck with cargo flamegraph on Mac due to SIP and other
system permissions problems.

Using Instruments on macOS works pretty well:
- Run the app.
- open -a "Instruments"
- Select one of the profilers. I used the CPU Profiler to find PNG loading
  slowness.
- Select the app from the Running Apps list.



