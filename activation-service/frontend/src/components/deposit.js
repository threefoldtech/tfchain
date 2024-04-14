import React, { useState } from 'react'
import { FormControl, InputLabel, Input, FormHelperText, Button } from '@material-ui/core'
import { Keyring } from '@polkadot/keyring'

export function ActivateAccount ({ activate }) {
  const [tfchainAddress, setTfchainAddress] = useState('')
  const [TfchainAddressError, setTfchainAddressError] = useState('')

  const submit = async () => {
    if (tfchainAddress === '') {
      setTfchainAddressError('Address not valid')
      return
    }

    try {
      const keyRing = new Keyring()
      keyRing.addFromAddress(tfchainAddress)
      // TODO CHECK IF ADDRESS IS VALID

      // if (!includes) {
      // setTfchainAddressError('Address does not have a valid trustline to TFT')
      // return
      // }
    } catch (error) {
      setTfchainAddressError('Address not found')
      return
    }

    setTfchainAddressError('')

    activate(tfchainAddress)
  }

  const handleTfchainAddressChange = (e) => {
    setTfchainAddressError('')
    setTfchainAddress(e.target.value)
  }

  return (
    <div>
      <div style={{ padding: '50px', display: 'flex', flexDirection: 'column', width: '90%', margin: 'auto' }}>
        <span style={{ fontSize: 22, marginBottom: '1em' }}>Enter your tfchain address to activate</span>
        <FormControl>
          <InputLabel htmlFor='tfchainAddress'>Tfchain Address</InputLabel>
          <Input
            value={tfchainAddress}
            onChange={handleTfchainAddressChange}
            id='tfchainAddress'
            aria-describedby='my-helper-text'
          />
          <FormHelperText id='my-helper-text'>Enter a tfchain address</FormHelperText>
          {TfchainAddressError && (
            <div>{TfchainAddressError}</div>
          )}
        </FormControl>

        <Button
          color='primary'
          variant='contained'
          style={{ marginTop: 25 }}
          type='submit'
          onClick={() => submit()}
        >
          Activate
        </Button>
      </div>
    </div>
  )
}
