<!--

   !!!!!!  WARNING: DO NOT EDIT THIS FILE!

   This file was generated from the Varnish VMOD source code.
   It will be automatically updated on each build.

-->
# Varnish Module (VMOD) `timestamp`

Measure time in VCL

```vcl
// Place import statement at the top of your VCL file
// This loads vmod from a standard location
import timestamp;

// Or load vmod from a specific file
import timestamp from "path/to/libtimestamp.so";
```

### Function `DURATION timestamp()`

Returns the duration since the same function was called for the last time (in the same task).
If it's the first time it's been called, return 0.

There could be only one type of per-task shared context data type in a Varnish VMOD.