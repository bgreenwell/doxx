# VHS Demo for doxx - Color Showcase
# Requires VHS: https://github.com/charmbracelet/vhs

# Output settings
Output "assets/demo.gif"
Set Width 900
Set Height 557
Set FontSize 14
Set Theme "Dracula"
Set TypingSpeed 50ms

# Show the command we're about to run
Type "/Users/bgreenwell/.cargo/bin/doxx tests/fixtures/color-showcase.docx --color"
Sleep 1s
Enter

# Wait for the application to load
Sleep 2s

# Scroll through the document to show different colored content
# Pause at the beginning to show the first content
Sleep 1s

# Scroll down one line at a time with half second pauses
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms
Down
Sleep 500ms

# Final pause before ending
Sleep 2s

# Exit the application
Type "q"
Sleep 500ms