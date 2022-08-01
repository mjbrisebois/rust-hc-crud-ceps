const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});

global.WebSocket			= require('ws');
const { AgentClient,
	HoloHashTypes }			= require('@whi/holochain-client');
const { Architecture, EntityType }	= require('@whi/entity-architect');
const { HoloHash,
	HoloHashError }			= HoloHashTypes;


const PostEntity			= new EntityType("post");
PostEntity.model("entry", content => {
    content.published_at	= new Date( content.published_at );
    content.last_updated	= new Date( content.last_updated );

    return content;
});

const CommentEntity			= new EntityType("comment");
CommentEntity.model("entry", content => {
    content.for_post		= new HoloHash( content.for_post );
    content.published_at	= new Date( content.published_at );
    content.last_updated	= new Date( content.last_updated );

    return content;
});

const schema				= new Architecture([ PostEntity, CommentEntity ]);


const all_clients			= [];
function exit_cleanup () {
    all_clients.forEach( client => client.close() );
}
process.once("exit", exit_cleanup );


class Client extends AgentClient {
    async callEntity ( ...args ) {
	let resp			= await this.call( ...args );

	try {
	    return schema.deconstruct( "entity", resp );
	} catch (err) {
	    console.log( err );
	}
    }

    async callCollection ( ...args ) {
	let resp			= await this.call( ...args );

	try {
	    return resp.map( entity => schema.deconstruct( "entity", entity ) );
	} catch (err) {
	    console.log( err );
	}
    }

    async call ( ...args ) {
	let resp			= await super.call( ...args );

	try {
	    return resp instanceof Uint8Array
		? new HoloHash( resp )
		: resp;
	} catch (err) {
	    if ( !(err instanceof HoloHashError) )
		console.log( err );
	    // else; Assume this is fine.  The response must not be a HoloHash
	}

	return resp;
    }
}


async function backdrop ( holochain, dnas, actors, client_options ) {
    log.normal("Setting up backdrop with %s DNAs and %s Agents", Object.keys(dnas).length, actors.length );

    log.debug("Adding stdout/stderr line event logging hooks");
    holochain.on("conductor:stdout", (line, parts) => {
	log.debug( "\x1b[39;1mConductor STDOUT:\x1b[22;37m %s", line );
    });

    log.debug("Waiting for holochain to start...");
    await holochain.start();

    const app_id			= "test";
    const app_port			= 44910;
    const clients			= {};

    log.debug("Waiting for DNAs and actors to be set up...");
    const agents			= await holochain.backdrop( app_id, app_port, dnas, actors );

    log.debug("Creating clients actors: %s", actors.join(", ") );
    await Promise.all( Object.entries( agents ).map( async ([ actor, happ ]) => {
	const dna_map			= {};
	await Promise.all( Object.entries( happ.cells ).map( async ([ nick, cell ]) => {
	    dna_map[nick]		= cell.dna.hash;
	    log.info("Established a new cell for '%s': %s => [ %s :: %s ]", actor, nick, String(cell.dna.hash), String(happ.agent) );
	}) );

	const client			= new Client( happ.agent, dna_map, app_port, client_options );
	clients[actor]			= client

	all_clients.push( client );
    }) );
    log.info("Finished backdrop setup: %s", Object.keys(clients) );

    return clients;
}


module.exports = {
    backdrop,
};
