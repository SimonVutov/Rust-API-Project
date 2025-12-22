const API = 'http://127.0.0.1:8080'

const notesEl = document.getElementById('notes')
const statusEl = document.getElementById('status')
const contentEl = document.getElementById('content')
const pinnedEl = document.getElementById('pinned')
const tagsEl = document.getElementById('tags')
const addBtn = document.getElementById('add')
const refreshBtn = document.getElementById('refresh')

function setStatus(msg, isError = false) {
  statusEl.textContent = msg
  statusEl.classList.toggle('error', isError)
}

function parseTags(value) {
  return value
    .split(',')
    .map((t) => t.trim())
    .filter((t) => t.length > 0)
}

function renderNote(note) {
  const li = document.createElement('li')
  li.className = 'note'

  const header = document.createElement('div')
  header.className = 'note-head'

  const meta = document.createElement('div')
  meta.className = 'meta'
  meta.textContent = `#${note.id} · ${new Date(Number(note.updated_ms)).toLocaleString()}`

  const pinBtn = document.createElement('button')
  pinBtn.className = note.pinned ? 'pill active' : 'pill'
  pinBtn.textContent = note.pinned ? 'Pinned' : 'Pin'
  pinBtn.addEventListener('click', () => togglePin(note))

  header.appendChild(meta)
  header.appendChild(pinBtn)

  const body = document.createElement('p')
  body.textContent = note.content || '(empty)'

  const tags = document.createElement('div')
  tags.className = 'tags'
  tags.textContent = note.tags && note.tags.length ? note.tags.join(', ') : 'no tags'

  const actions = document.createElement('div')
  actions.className = 'actions'

  const del = document.createElement('button')
  del.className = 'danger'
  del.textContent = 'Delete'
  del.addEventListener('click', () => deleteNote(note))
  actions.appendChild(del)

  const edit = document.createElement('button')
  edit.className = 'secondary'
  edit.textContent = 'Edit'
  edit.addEventListener('click', () => editNote(note))
  actions.appendChild(edit)

  const view_changes = document.createElement('button')
  view_changes.className = 'secondary'
  view_changes.textContent = 'Changes'
  view_changes.addEventListener('click', () => viewChanges(note))
  actions.appendChild(view_changes)

  li.appendChild(header)
  li.appendChild(body)
  li.appendChild(tags)
  li.appendChild(actions)
  return li
}

async function fetchNotes() {
  setStatus('Loading...')
  try {
    const res = await fetch(`${API}/api/notes`)
    const data = await res.json()
    notesEl.innerHTML = ''
    data.forEach((note) => notesEl.appendChild(renderNote(note)))
    setStatus(`${data.length} notes`)
  } catch (err) {
    setStatus('Failed to load notes', true)
  }
}

async function addNote() {
  const payload = {
    content: contentEl.value.trim(),
    pinned: pinnedEl.checked,
    tags: parseTags(tagsEl.value),
  }

  if (!payload.content) {
    setStatus('Content is empty', true)
    return
  }

  setStatus('Saving...')
  try {
    const res = await fetch(`${API}/api/notes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    })
    if (!res.ok) {
      throw new Error('Bad response')
    }
    contentEl.value = ''
    tagsEl.value = ''
    pinnedEl.checked = false
    await fetchNotes()
  } catch (err) {
    setStatus('Failed to save note', true)
  }
}

async function togglePin(note) {
  setStatus('Updating...')
  try {
    const res = await fetch(`${API}/api/notes/${note.id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pinned: !note.pinned }),
    })
    if (!res.ok) {
      throw new Error('Bad response')
    }
    await fetchNotes()
  } catch (err) {
    setStatus('Failed to update note', true)
  }
}

async function deleteNote(note) {
  setStatus('Deleting...')
  try {
    const res = await fetch(`${API}/api/notes/${note.id}`, { method: 'DELETE' })
    if (!res.ok && res.status !== 204) {
      throw new Error('Bad response')
    }
    await fetchNotes()
  } catch (err) {
    setStatus('Failed to delete note', true)
  }
}

async function viewChanges(note) {
  try {
    const res = await fetch(`${API}/api/notes-changes/${encodeURIComponent(note.id)}`)
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}`)
    }
    const text = await res.text()
    alert(text || '(no changes)')
  } catch (err) {
    setStatus('Failed to load changes', true)
  }
}

async function editNote(note) {
  const newContent = prompt('Edit note content:', note.content)
  if (newContent === null) {
    return
  }

  const newTags = prompt('Edit note tags (comma separated):', note.tags.join(', '))
  if (newTags === null) {
    return
  }

  setStatus('Updating...')
  try {
    const res = await fetch(`${API}/api/notes/${note.id}`, {
      method: 'PATCH',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        content: newContent.trim(),
        tags: parseTags(newTags),
      }),
    })
    if (!res.ok) {
      throw new Error('Bad response')
    }
    await fetchNotes()
  } catch (err) {
    setStatus('Failed to update note', true)
  }
}

addBtn.addEventListener('click', addNote)
refreshBtn.addEventListener('click', fetchNotes)

fetchNotes()
