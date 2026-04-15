# Bookmarks demo
Output docs/gifs/bookmarks.gif

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

Type "kubernetes"
Enter
Sleep 300ms
Enter
Sleep 3s

# Navigate to first result
Down
Down
Sleep 500ms

# Bookmark it
Type "b"
Sleep 1s

# Open bookmark list
Ctrl+b
Sleep 1s

# Navigate within list
Down
Sleep 500ms

# Copy URL
Type "y"
Sleep 800ms

# Close
Escape
Sleep 500ms

Type "q"
