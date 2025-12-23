const API = 'http://127.0.0.1:8080'

const notesEl = document.getElementById('notes')
const statusEl = document.getElementById('status')
const contentEl = document.getElementById('content')
const pinnedEl = document.getElementById('pinned')
const tagsEl = document.getElementById('tags')
const addBtn = document.getElementById('add')
const refreshBtn = document.getElementById('refresh')
const usernameEl = document.getElementById('username')
const passwordEl = document.getElementById('password')
const signoutEl = document.getElementById('signout')

const signupEl = document.getElementById('signup')
const signinEl = document.getElementById('signin')

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
    const res = await fetch(`${API}/api/notes`, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${localStorage.getItem('sessionToken')}` },
    })
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
      body: JSON.stringify({ ...payload, session_token: localStorage.getItem('sessionToken') }),
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
      body: JSON.stringify({ pinned: !note.pinned, session_token: localStorage.getItem('sessionToken') }),
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
    const res = await fetch(`${API}/api/notes/${note.id}`, {
      method: 'DELETE',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_token: localStorage.getItem('sessionToken') }),
    })
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
    const res = await fetch(`${API}/api/notes-changes/${encodeURIComponent(note.id)}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ session_token: localStorage.getItem('sessionToken') }),
    })
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
        session_token: localStorage.getItem('sessionToken'),
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

function hideSignupSignin() {
  signupEl.style.display = 'none'
  signinEl.style.display = 'none'
  usernameEl.style.display = 'none'
  passwordEl.style.display = 'none'
  signoutEl.style.display = 'inline-block'
}

function showSignupSignin() {
  signupEl.style.display = 'inline-block'
  signinEl.style.display = 'inline-block'
  usernameEl.style.display = 'inline-block'
  passwordEl.style.display = 'inline-block'
  signoutEl.style.display = 'none'
}

async function authenticate(signin_or_signup = 'signup') {
  const username = document.getElementById('username').value.trim()
  const password = document.getElementById('password').value.trim()

  if (!username || !password) {
    setStatus('Username and password are required', true)
    return
  }

  if (signin_or_signup === 'signin') {
    setStatus('Signing  in...')
    try {
      const res = await fetch(`${API}/api/signin`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password, session_token: localStorage.getItem('sessionToken') }),
      })
      if (!res.ok) {
        throw new Error('Bad response, Error: ' + res.status)
      }
      const data = await res.json()
      localStorage.setItem('sessionToken', data.session_token)

      setStatus('Signin successful!')
      fetchNotes()
    } catch (err) {
      setStatus('Failed to sign in, Error: ' + err.message, true)
    }
  } else {
    setStatus('Signing up...')
    try {
      const res = await fetch(`${API}/api/signup`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username, password, session_token: localStorage.getItem('sessionToken') }),
      })
      // if res is 500, it is likely that they meant to sign in instead
      if (res.status === 500) {
        throw new Error('User may already exist. Please try signing in instead.')
      }
      if (!res.ok) {
        throw new Error('Bad response, Error: ' + res.status)
      }
      setStatus('Signup successful! Now signing you in...')
      authenticate('signin')
    } catch (err) {
      setStatus('Failed to sign up, Error: ' + err.message, true)
    }
  }
  if (localStorage.getItem('sessionToken')) hideSignupSignin()
  else showSignupSignin()
}

async function signout() {
  localStorage.removeItem('sessionToken')
  setStatus('Signed out')
  showSignupSignin()
  fetchNotes()
}

addBtn.addEventListener('click', addNote)
refreshBtn.addEventListener('click', fetchNotes)
signupEl.addEventListener('click', () => authenticate('signup'))
signinEl.addEventListener('click', () => authenticate('signin'))
signoutEl.addEventListener('click', signout)

fetchNotes()
showSignupSignin()
if (localStorage.getItem('sessionToken')) hideSignupSignin()
