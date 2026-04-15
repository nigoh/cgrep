# Search bar demo: tag input + AND/OR toggle
Output docs/gifs/search-bar.gif

Set Shell "bash"
Set FontSize 14
Set Width 120
Set Height 20
Set Theme "Dracula"

# Setup env
Hide
  Type "export CONFLUENCE_URL=https://c.example.com CONFLUENCE_USER=u CONFLUENCE_TOKEN=t JIRA_URL=https://j.example.com JIRA_USER=u JIRA_TOKEN=t GERRIT_URL=https://g.example.com GERRIT_USER=u GERRIT_PASSWORD=p"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

# Type a keyword
Type "kubernetes"
Sleep 800ms
Enter
Sleep 400ms

# Type another
Type "deployment"
Sleep 800ms
Enter
Sleep 600ms

# Toggle to OR
Ctrl+m
Sleep 800ms

# Toggle back to AND
Ctrl+m
Sleep 800ms

# Delete last tag
Backspace
Sleep 800ms

# Quit
Type "q"
