# GlazeWM Architecture

This document outlines the key architectural decisions and patterns used in GlazeWM, with a focus on coordinate space handling and multi-monitor support.

## Overview

GlazeWM follows a container-based architecture where the window manager maintains a tree of containers representing monitors, workspaces, and windows. The tiling pipeline flows from input events through the layout engine to final window placement.

## Coordinate Spaces and DPI Handling

### Coordinate Space Strategy

GlazeWM operates primarily in **physical pixels** throughout the system to ensure consistent behavior across mixed-DPI environments.

- **Source of Truth**: Monitor bounds and working areas are obtained via Windows APIs (`GetMonitorInfoW`, `GetDpiForMonitor`) in physical pixels
- **Conversion Policy**: No coordinate space conversions are performed internally; all calculations use physical pixels
- **DPI Awareness**: The application uses per-monitor DPI awareness (PMv2) to handle mixed-resolution setups correctly

### Monitor Topology

#### Detection and Normalization

1. **Monitor Enumeration**: Uses `EnumDisplayMonitors` to discover available displays
2. **Bounds Calculation**: Each monitor's `rect` and `working_rect` are stored in physical pixels
3. **DPI Information**: Scale factors are calculated as `dpi / 96.0` for each monitor
4. **Multi-Monitor Coordinates**: The system respects Windows' virtual desktop coordinate space where monitors can have different origins

#### Edge/Corner Handling

- **Monitor Boundaries**: Windows are constrained to their assigned monitor's working area
- **Cross-Monitor Operations**: Window movements between monitors account for DPI differences
- **Spillover Prevention**: Active clamping prevents windows from extending onto adjacent monitors

## Window Placement Pipeline

### Tiling Calculation Flow

1. **Layout Engine**: Computes ideal window rectangles based on container tree structure
2. **Border Application**: Applies border deltas for visual effects and gaps
3. **Monitor Clamping**: Constrains final rectangles to monitor working areas using `clamp_within_bounds()`
4. **Platform Sync**: Calls `SetWindowPos` with validated coordinates

### Rect Computation and Clamping

The core placement logic in `platform_sync.rs`:

```rust
let original_rect = window.to_rect()?.apply_delta(&window.total_border_delta()?, None);
let monitor = window.monitor()?;
let working_rect = monitor.native().working_rect()?;
let rect = original_rect.clamp_within_bounds(working_rect);
```

#### Clamping Rules

- **Positioning**: Windows that would extend beyond monitor bounds are repositioned
- **Sizing**: Windows larger than the monitor are resized to fit
- **Priority**: Preserves original dimensions when possible, repositions as needed

### DPI Adjustment Strategy

#### Process DPI Awareness

- **Context**: Uses `SetThreadDpiAwarenessContext` for per-monitor awareness
- **Window Queries**: All coordinate queries (`GetWindowRect`, `DwmGetWindowAttribute`) respect current DPI context
- **Coordinate Validation**: Final coordinates are validated to be within reasonable bounds (-32768 to 32767)

#### Multi-Step Positioning

For windows with pending DPI adjustments:

1. **Initial Positioning**: Standard `SetWindowPos` call with calculated rectangle
2. **DPI Adjustment**: Second `SetWindowPos` call to resolve scaling inconsistencies
3. **Bounds Validation**: Coordinates are validated before the second call

## Testing Strategy

### Unit Tests

- **Rect Mathematics**: Parameterized tests for clamping operations across different monitor configurations
- **Boundary Conditions**: Tests for edge cases like oversized windows, negative coordinates
- **Mixed Resolution**: Specific scenarios covering 4K + 1920×1200 setups

### Integration Testing Approach

While full integration tests require actual hardware, the architecture supports:

- **Monitor Mocking**: `NativeMonitor` instances with configurable bounds and DPI
- **Window Simulation**: Mock windows with controllable positions and states
- **Pipeline Testing**: End-to-end placement logic with simulated monitor topologies

## Diagnostics and Debugging

### Structured Logging

When window clamping occurs, detailed geometry information is logged:

- Original and clamped rectangles
- Monitor working area bounds  
- DPI and scale factor information
- Window and monitor identifiers

### Debug Features

- **Geometry Tracing**: Structured logs track coordinate transformations
- **Monitor Information**: Runtime monitor topology and DPI information available via IPC
- **Window State**: Current window rectangles and placement decisions are traceable

## Common Patterns

### Error Handling

- **Fallback Behavior**: When monitor information is unavailable, operations gracefully degrade
- **Coordinate Validation**: Invalid coordinates are detected and corrected before Windows API calls
- **DPI Context**: DPI-related operations include error recovery and fallback strategies

### Performance Considerations

- **Lazy Evaluation**: Monitor information is cached and only refreshed when display settings change
- **Batch Operations**: Window updates are batched to minimize API calls
- **Async Operations**: Z-order updates use async timing to handle Windows API quirks

## Migration and Compatibility

### Backward Compatibility

- **Same-Resolution Setups**: Existing behavior is preserved for single-DPI configurations
- **Configuration**: No configuration changes required for the fix
- **API Stability**: All public interfaces remain unchanged

### Future Considerations

- **Logical Pixel Support**: Architecture can be extended to support logical pixel calculations if needed
- **Enhanced Validation**: Additional coordinate validation could be added without architectural changes
- **Cross-Platform**: The coordinate space strategy is Windows-specific but the pattern could extend to other platforms