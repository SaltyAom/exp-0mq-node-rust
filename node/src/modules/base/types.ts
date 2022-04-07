import type { RouteShorthandMethod } from 'fastify'

export interface UserHandler extends RouteShorthandMethod {
    Params: {
        user: string
    }
}
