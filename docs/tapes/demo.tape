# cgrep full demo
Output docs/gifs/demo.gif

Require cgrep

Set Shell "bash"
Set FontSize 14
Set Width 160
Set Height 45
Set Theme "Dracula"

Hide
  Type "export CONFLUENCE_URL=https://confluence.example.com"
  Enter
  Type "export CONFLUENCE_USER=demo@example.com"
  Enter
  Type "export CONFLUENCE_TOKEN=demo_token"
  Enter
  Type "export JIRA_URL=https://jira.example.com"
  Enter
  Type "export JIRA_USER=demo@example.com"
  Enter
  Type "export JIRA_TOKEN=demo_token"
  Enter
  Type "export GERRIT_URL=https://gerrit.example.com"
  Enter
  Type "export GERRIT_USER=demo"
  Enter
  Type "export GERRIT_PASSWORD=demo_pass"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

# Type keywords
Type "kubernetes"
Sleep 300ms
Enter
Sleep 200ms
Type "deployment"
Sleep 300ms
Enter
Sleep 3s

# Navigate results
Down
Down
Sleep 500ms
Down
Sleep 500ms

# Press p to preview
Type "p"
Sleep 1s

# Back to list
Escape
Sleep 500ms

# Press b to bookmark
Type "b"
Sleep 1s

# Open bookmarks
Ctrl+b
Sleep 1s
Escape
Sleep 500ms

# Open filter
Tab
Sleep 1s
Down
Down
Sleep 300ms
Space
Sleep 1s
Escape
Sleep 500ms

# New tab
Ctrl+t
Sleep 500ms
Type "monitoring"
Enter
Sleep 2s

# Switch back
Ctrl+Left
Sleep 500ms

# Quit
Type "q"
