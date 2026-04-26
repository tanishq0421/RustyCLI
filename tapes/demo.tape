# vhs demo.tape — produces docs/demo.gif
# Generate with:  vhs tapes/demo.tape

Output docs/demo.gif

Set FontSize 16
Set Width 1000
Set Height 600
Set Theme "Dracula"
Set TypingSpeed 30ms

Type "cargo run --quiet"  Enter
Sleep 1500ms

Type "echo Hello, RustyCLI!"  Enter
Sleep 600ms

Type "export NAME=tanishq"  Enter
Type "echo Hi $NAME"  Enter
Sleep 600ms

Type "ls src/*.rs | wc -l"  Enter
Sleep 800ms

Type "alias ll='ls -la'"  Enter
Type "ll | head -3"  Enter
Sleep 1000ms

Type "echo first > /tmp/demo.txt"  Enter
Type "echo second >> /tmp/demo.txt"  Enter
Type "cat /tmp/demo.txt"  Enter
Sleep 800ms

Type "exit"  Enter
Sleep 500ms
