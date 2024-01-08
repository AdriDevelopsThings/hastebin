var state = {}
var typingTimeout = null
var lastUploadedContent = null

const textarea = document.getElementById('textarea')
const filenameField = document.getElementById('filename')
const createNewButton = document.getElementById('create-new')

function main() {
    const path = window.location.pathname.split('/')
    if (path.length == 3) {
        const fileId = path[1]
        const fileName = path[2]
        filenameField.readOnly = true
        state.file_id = fileId
        state.file_name = fileName
        const change_key = sessionStorage.getItem(`change_key_${fileId}_${fileName}`)
        if (change_key) {
            state.change_key = change_key
        }
        fetchContent()
    }
}

function updateUriToState() {
    if (state.file_id && state.file_name) {
        const fileId = state.file_id
        const fileName = state.file_name
        filenameField.readOnly = true
        window.history.pushState({}, 'Hastebin', `/${fileId}/${fileName}`)
    } else {
        window.history.pushState({}, 'Hastebin', '/')
    }
}

async function fetchContent() {
    if (state.file_id && state.file_name) {
        const fileId = state.file_id
        const fileName = state.file_name
        const response = await fetch(`/api/file/${fileId}/${fileName}`)
        const text = await response.text()
        textarea.value = text
        if (!state.change_key) {
            textarea.readOnly = true
        }
    }
}

async function deleteHaste() {
    if (state.file_id && state.file_name && state.change_key) {
        const fileId = state.file_id
        const fileName = state.file_name
        fetch(`/api/file/${fileId}/${fileName}`, {
            method: 'DELETE',
            headers: { 'Change-Key': state.change_key }
        })
        lastUploadedContent = null
        state = {}
        updateUriToState()
        textarea.value = ''
    }
}

async function uploadContent() {
    const content = textarea.value
    if (lastUploadedContent == content || (lastUploadedContent == null && !content)) { // textarea is empty
        return
    }
    if (state.file_id && state.file_name) {
        if (!state.change_key) {
            return
        }
        if (!content) {
            return deleteHaste()
        }
        const fileId = state.file_id
        const fileName = state.file_name
        await fetch(`/api/file/${fileId}/${fileName}`, {
            method: 'PUT',
            body: content,
            headers: { 'Change-Key': state.change_key }
        })
    } else {
        // create new file
        const fileName = filenameField.value
        const response = await fetch(`/api/file/${fileName}`, { method: 'POST', body: content })
        const json = await response.json()
        const fileId = json.id
        state.file_name = fileName
        state.file_id = fileId
        state.change_key = json.change_key
        sessionStorage.setItem(`change_key_${fileId}_${fileName}`, json.change_key)
        updateUriToState()
    }
    lastUploadedContent = content
}

window.addEventListener('keydown', event => {
    // ctrl + s
    if ((navigator.userAgentData.platform.match("Mac") ? event.metaKey : event.ctrlKey) && event.key == 's') {
        event.preventDefault()
        uploadContent()
    }
})

textarea.addEventListener('focusout', uploadContent)
textarea.addEventListener('keydown', () => {
    if (typingTimeout) {
        clearTimeout(typingTimeout)
    }
})
textarea.addEventListener('keyup', () => {
    clearTimeout(typingTimeout)
    typingTimeout = setTimeout(uploadContent, 3000)
})

createNewButton.addEventListener('click', () => {
    clearTimeout(typingTimeout)
    state = {}
    updateUriToState()
    textarea.value = ''
    filenameField.readOnly = false
    filenameField.value = 'text.txt'
})

main()