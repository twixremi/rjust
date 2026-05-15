# rjust Framework

`rjust` is a high-performance, low-level modding framework and runtime environment designed for JVM applications (such as Minecraft). It decouples heavy computation from the JVM garbage collector by orchestrating native execution threads in isolated sandboxes.

## Architecture

The framework splits execution into two standalone boundaries connected via IPC (Inter-Process Communication):

1. **Java Agent:** A lightweight agent injected into the host JVM process to handle memory mapping, primitive state synchronization, and execution hooks.
2. **Rust Worker:** A fully native, isolated process that consumes shared memory data, executes complex algorithms (kinematics, ray tracing, OS-level inputs), and operates within a safe system sandbox.

By moving hot code paths from Java to Rust, `rjust` eliminates micro-stutters caused by Garbage Collection (GC) pauses and allows multi-threaded scheduling that completely bypasses the main JVM thread bottleneck.

## Ecosystem & Building

This project utilizes the **Crow Build System** for module compilation, incremental task runner management, and offline dependency provisioning. 
or just use cargo :) but i recommend use my builder /twixremi/cbs

To build `rjust` and its sample modifications, ensure you have the `crow` binary installed and run:

```bash
crow build
