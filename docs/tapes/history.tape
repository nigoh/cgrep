# Search history demo
Output docs/gifs/history.gif

Set Shell "bash"
Set FontSize 14
Set Width 140
Set Height 35
Set Theme "Dracula"

Hide
  Type "export CONFLUENCE_URL=https://c.example.com CONFLUENCE_USER=u CONFLUENCE_TOKEN=t JIRA_URL=https://j.example.com JIRA_USER=u JIRA_TOKEN=t GERRIT_URL=https://g.example.com GERRIT_USER=u GERRIT_PASSWORD=p"
  Enter
Show

Type "cgrep"
Enter
Sleep 500ms

# First search
Type "kubernetes"
Enter
Sleep 200ms
Enter
Sleep 2s

# Second search (new tab)
Ctrl+t
Sleep 400ms
Type "monitoring"
Enter
Sleep 200ms
Enter
Sleep 2s

# Open history
Up
Sleep 1s

# Navigate
Down
Sleep 500ms

# Restore history entry
Enter
Sleep 1s

Type "q"
