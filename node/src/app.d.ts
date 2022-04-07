/* eslint-disable @typescript-eslint/no-unused-vars */
import type fastify, { FastifyRequest } from 'fastify'

import type { Router } from 'zeromq'

declare module 'fastify' {
    interface FastifyRequest {
        router: Router
    }
}