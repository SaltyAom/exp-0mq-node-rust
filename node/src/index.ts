import env from 'dotenv'
env.config()

import fastify from 'fastify'

import helmet from 'fastify-helmet'
import staticPlugin from 'fastify-static'

import { resolve } from 'path'

import { base } from '@modules'
import { push, pull, createPool } from '@services'

const app = fastify()

const main = async () => {
    await Promise.all([
        push.connect('tcp://0.0.0.0:5555'),
        pull.bind('tcp://0.0.0.0:5556')
    ])

    createPool(pull)

    app.register(helmet)
        .register(staticPlugin, {
            root: resolve('./public')
        })
        .register(base)
        .listen(8080, '0.0.0.0', (error, address) => {
            if (error) return console.error(error)

            console.log(`Running at ${address}`)
        })
}

main()
