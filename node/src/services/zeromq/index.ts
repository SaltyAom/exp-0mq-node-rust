/* eslint-disable no-constant-condition */
import { Push, Pull } from 'zeromq'
import { nanoid } from 'nanoid'
import { EventEmitter } from 'stream'

export const push = new Push()
export const pull = new Pull()

const events = new EventEmitter()
events.setMaxListeners(Infinity)

export const createPool = async (pull: Pull) => {
    while (true) events.emit('buffer', (await pull.receive())[0])
}

const splitAt = (index: number, xs: string) => [
    xs.slice(0, index),
    xs.slice(index)
]

export const scylla = async (message: string) => {
    const id = nanoid()

    await push.send(`${id}${message}`)

    return new Promise<string>((resolve, reject) => {
        setTimeout(reject, 5000)

        events.addListener('buffer', (buffer: Buffer) => {
            const [localId, message] = splitAt(21, buffer.toString())

            if (localId === id) resolve(message.toString())
        })
    })
}

process.on('exit', () => {
    push.close()
    pull.close()
})
