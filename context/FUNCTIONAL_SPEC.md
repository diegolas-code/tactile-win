# Functional Specification – Grid-Based Window Management

This document describes the functional behavior of the application and is intended to serve as context and specification for development and design decisions.

---

## Grid Definition

The application divides the screen(s) into a grid composed of N rows and M columns, defined by the user through a configuration menu.  
These values are constrained by developer-defined thresholds that depend on the resolution of the screen(s).

For example, each grid cell must not be smaller than a minimum width and height measured in pixels.

---

## Grid Layout and Labeling

Each screen is divided into a regular grid, for example 4 columns (width) × 2 rows (height).

Each grid cell is labeled with a letter following the user's physical keyboard layout, typically QWERTY. The maximum number of columns corresponds to the number of letter keys in the keyboard row that starts with the letter Q, and the maximum number of rows corresponds to the number of letter rows on the keyboard (typically three).

For example, in a 4×2 grid:

- The four cells of the first row are labeled Q, W, E, R
- The four cells of the second row are labeled A, S, D, F

---

## Window Selection Interaction

When the global keyboard shortcut that activates the application is executed, a grid overlay displaying the labeled cells is shown on top of the screen(s).

The user determines the size and position of the currently active window by typing the letters corresponding to the grid cells the window should occupy.

For example, in a 4×2 grid:

- The user activates the application.
- The grid overlay appears.
- The user types the letters Q and S.
- The active window is resized to occupy the cells Q, W, A, S.

---

## Multi-Monitor Selection

To choose on which screen the active window should be repositioned, while the grid overlay is visible on all screens, the user navigates between screens using the left and right arrow keys.

The user may select grid coordinates that belong to two different screens, allowing the active window to span across monitors or be moved between them.

---

## Overlay Legend

When the grid overlay is displayed, a small legend is also shown.  
This legend indicates:

- How to access on-screen help.
- How to open the application configuration.

---

## Functional Requirements

The application must:

- Detect the number of connected screens.
- Detect the resolution of each screen.
- Initialize each screen with a default grid size of 3×2.
- Prevent the user from configuring a grid in which any cell would be smaller than 480×360 pixels.

---
