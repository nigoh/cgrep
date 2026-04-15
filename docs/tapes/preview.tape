# Preview panel demo
Output docs/gifs/preview.gif

Set Shell "bash"
Set FontSize 14
Set Width 160
Set Height 40
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

# Navigate results to show preview updating
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms
Down
Sleep 600ms

# Focus preview
Type "p"
Sleep 800ms

# Scroll preview
Down
Sleep 400ms
Down
Sleep 400ms
Up
Sleep 400ms

# Back to list
Escape
Sleep 500ms

Type "q"
