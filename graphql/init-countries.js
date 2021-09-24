require('dotenv').config()
const { Pool } = require('pg')
const axios = require('axios')
const { map, flatten } = require('lodash')
const format = require('pg-format')

const {
    DB_HOST,
    DB_NAME,
    DB_PASS,
    DB_PORT,
    DB_USER
} = process.env

async function main () {
    const config = {
        user: DB_USER,
        password: DB_PASS,
        port: DB_PORT,
        host: DB_HOST,
        database: DB_NAME
    }

    const countries = await getCountries()

    const cities = await getCities()
    
    const pool = new Pool(config)
    pool.on('error', (err, client) => {
        console.error('Unexpected error on idle client', err)
        process.exit(-1)
    })
    
    const client = await pool.connect()

    try {
        const countryPromises = countries.data.map((country, index) => {
            const text = 'INSERT INTO country(id, country_id, name, code, region, subregion, lat, long, created_by_id, version) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)'
            let code = country.alpha2Code
            if (!code) {
                code = country.alpha3Code
            }

            let lat, long = ''
            if (country.latlng) {
                lat = country.latlng[0].toString()
                long = country.latlng[1].toString()
            }
            index++
            
            const region = country.continent || "unknown region"
            const subregion = country.region || "unknown subregion"

            return client.query(text, [index, index, country.name, code, region, subregion, lat, long, 0, 0]) 
        })
    
        // await Promise.all(countryPromises)

        const query = {
            name: 'fetch',
            text: 'select * from country'
        }

        const res = await client.query(query)
        const countryData = res.rows

        let index = 0
        const mappedCities = map(cities.data, (countryCities, country) => {
            let foundCountry = countryData.filter(c => c.name === country)[0]
            if (!foundCountry) return

            foundCountryID = foundCountry.country_id || foundCountry.id

            return countryCities.map(countryCity => {
                if (countryCity === '') {
                    countryCity = 'Unknown'
                }

                const text = 'INSERT INTO city(id, city_id, country_id, name, created_by_id, version) VALUES($1, $2, $3, $4, $5, $6) RETURNING *'
                index++
               
                return [index, index, foundCountryID, countryCity, 0, 0]
            })
        }).filter(g => g)

        const inserts = format('INSERT INTO city(id, city_id, country_id, name, created_by_id, version) VALUES %L', flatten(mappedCities))

        // console.log(inserts)
        client.query(inserts)
            .then(res => {
                console.log(res)
            })
            .catch(err => console.log(err))
            .then(process.exit(0))

    } catch (error) {
        console.log(error)
        process.exit(1)
    }
}

async function getCountries () {
    return axios.get('https://restcountries.com/v2/all')
}

async function getCities () {
    return axios.get('https://raw.githubusercontent.com/shivammathur/countrycity/master/data/geo.json')
}

main()