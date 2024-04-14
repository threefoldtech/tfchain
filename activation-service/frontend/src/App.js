import logo from './3fold_logo.png'
import './App.css'
import { ActivateAccount } from './components/deposit'
import axios from 'axios'
import { useSnackbar } from 'notistack'

const URL = process.env.NODE_ENV === 'development' ? 'http://localhost:3000' : ''

function App () {
  const { enqueueSnackbar } = useSnackbar()

  const handleActivate = (account) => {
    console.log(URL)
    axios.post(`${URL}/activation/activate`, { substrateAccountID: account })
      .then(res => {
        enqueueSnackbar('Successfully activated account!')
        console.log(res)
      })
      .catch(err => {
        enqueueSnackbar('Failed to activate account')
        console.log(err)
      })
  }

  return (
    <div className='App'>
      <header className='App-header'>
        <img src={logo} className='App-logo' alt='logo' />
        <ActivateAccount activate={handleActivate} />
      </header>
    </div>
  )
}

export default App
