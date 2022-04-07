import type { FastifyPluginCallback } from 'fastify'
import { scylla } from '@services'

import type { UserHandler } from './types'

const base: FastifyPluginCallback = (app, _, done) => {
    app.get('/', async (_req, res) => {
        res.send('Working')
    })

    app.get<UserHandler>('/user/:user', async (req, res) => {
        const {
            params: { user }
        } = req

        try {
            const res = await scylla(user)
            if (!res) return "Couldn't find user"

            return res
        } catch (_) {
            res.status(408)
        }
    })

    done()
}

export default base
