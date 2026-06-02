function hex2a (hex) {
  // Guard against absent/null fields. On the current runtime several previously
  // byte-encoded fields are Option<...> (decoded as null) or were removed
  // entirely, so callers may pass undefined/null. Returning '' keeps the read
  // wrappers from throwing `Cannot read properties of undefined`.
  if (hex === undefined || hex === null) return ''
  let str = ''
  for (let i = 0; i < hex.length; i += 2) {
    const v = parseInt(hex.substr(i, 2), 16)
    if (v) str += String.fromCharCode(v)
  }
  return str
}

function validateID (id) {
  try {
    parseInt(id)
  } catch (error) {
    throw Error('ID must be an integer')
  }
  if (isNaN(id) || id === 0) {
    throw Error('You must pass a valid ID')
  }
}

module.exports = { hex2a, validateID }
