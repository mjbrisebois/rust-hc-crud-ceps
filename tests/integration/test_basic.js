const path				= require('path');
const log				= require('@whi/stdlog')(path.basename( __filename ), {
    level: process.env.LOG_LEVEL || 'fatal',
});


const fs				= require('fs');
const expect				= require('chai').expect;
const { Holochain }			= require('@whi/holochain-backdrop');
const { RibosomeError }			= require('@whi/holochain-client');
const json				= require('@whi/json');
// const why				= require('why-is-node-running');

// setTimeout(() => {
//     console.log( why() );
// }, 6000 );

const { backdrop }			= require('./setup.js');

const delay				= (n) => new Promise(f => setTimeout(f, n));
const DNA_PATH				= path.join( __dirname, "../dnas/happy_path.dna" );


async function expect_reject ( cb, error, message ) {
    let failed				= false;
    try {
	await cb();
    } catch (err) {
	failed				= true;
	expect( () => { throw err }	).to.throw( error, message );
    }
    expect( failed			).to.be.true;
}


let clients;
let post, post2;
let comment, comment2;
let create_post_input			= {
    "message": "Hello, world!",
};
let create_comment_input_1		= {
    "message": "Don't call me surely",
};
let create_comment_input_2		= {
    "message": "I've never been on a plane before",
};


function basic_tests () {
    it("should test 'create_entity'", async function () {
	post				= await clients.alice.callEntity( "happy_path", "happy_path", "create_post", create_post_input );
	// console.log( json.debug(post) )

	expect( post.message		).to.equal( create_post_input.message );

	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "create_post", create_post_input );
	// console.log( json.debug(post2) )
    });

    it("should test 'get_entity'", async function () {
	post				= await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
	    "id": post.$id,
	});
	// console.log( json.debug(post) )

	expect( post.message		).to.equal( create_post_input.message );
    });

    it("should test 'update_entity'", async function () {
	let input			= Object.assign( {}, create_post_input, {
	    "message": "Goodbye, world!",
	});

	let prev_post			= post2;
	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "update_post", {
	    "addr": post2.$action,
	    "properties": input,
	});
	// console.log( json.debug(post2) )

	expect( post2.message		).to.equal( input.message );
	expect( post2.$action		).to.not.deep.equal( prev_post.$action );

	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
	    "id": post2.$id,
	});
	// console.log( json.debug(post2) )

	expect( post2.message		).to.equal( input.message );
	expect( post2.$action		).to.not.deep.equal( prev_post.$action );
    });

    it("should test 'Collection'", async function () {
	this.timeout( 5_000 );
	{
	    create_comment_input_1.for_post = post.$id;
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post.$id,
		"comment": create_comment_input_1,
	    });

	    expect( comment.message		).to.equal( create_comment_input_1.message );
	    expect( comment.for_post		).to.deep.equal( post.$id );

	    create_comment_input_2.for_post = post2.$id;
	    comment2			= await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post2.$id,
		"comment": create_comment_input_2,
	    });
	}

	{
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "get_comment", {
		"id": comment.$id,
	    });

	    expect( comment.message		).to.equal( create_comment_input_1.message );
	    expect( comment.for_post		).to.deep.equal( post.$id );
	}

	{
	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 1 );
	}

	{
	    let input			= Object.assign( {}, create_comment_input_1, {
		"message": "I just want to tell you both, good luck. We're all counting on you.",
	    });

	    let prev_comment		= comment;
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "update_comment", {
		"addr": comment.$action,
		"properties": input,
	    });

	    expect( comment.$action	).to.not.deep.equal( prev_comment.$action );

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 1 );
	    expect( comments[0].message	).to.equal( input.message );
	    expect( comments[0].$action	).to.not.deep.equal( prev_comment.$action );
	}

	{
	    await clients.alice.call( "happy_path", "happy_path", "link_comment_to_post", {
		"comment_id": comment.$id,
		"post_id": post2.$id,
	    });

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post2.$id );

	    expect( comments		).to.have.length( 2 );
	}

	{
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "move_comment_to_post", {
		"comment_addr": comment.$action,
		"post_id": post2.$id,
	    });

	    expect( comment.for_post	).to.not.deep.equal( post.$id );

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 0 );

	    let comments2		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post2.$id );

	    expect( comments2		).to.have.length( 2 );
	}

	{
	    let delete_hash		= await clients.alice.call( "happy_path", "happy_path", "delete_comment", {
		"id": comment.$id,
	    });

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post2.$id );

	    expect( comments		).to.have.length( 1 );
	}
    });

    it("should test 'delete_entity'", async function () {
	let delete_hash			= await clients.alice.call( "happy_path", "happy_path", "delete_post", {
	    "id": post.$id,
	});

	expect( delete_hash		).to.be.a("ActionHash");
    });
}

function errors_tests () {
    it("should fail to 'get_entity' because address is wrong entry type", async function () {
	await expect_reject( async () => {
	    let resp = await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
		"id": comment2.$id,
	    });
	}, RibosomeError, "Deserialized entry to wrong type: expected 0/0 but found 0/1" );
    });

    it("should fail to update because of wrong entry type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "update_comment", {
		"addr": post2.$action,
		"properties": create_comment_input_1,
	    });
	}, RibosomeError, "Failed to deserialize to entry type 'Comment'" );
    });

    it("should fail to update because mismatched type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "update_post", {
		"addr": comment2.$action,
		"properties": create_post_input,
	    });
	}, RibosomeError, "Deserialized entry to wrong type: expected 0/0 but found 0/1" );
    });

    it("should fail to create comment because post is deleted", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post.$id,
		"comment": create_comment_input_1,
	    });
	}, RibosomeError, "Record not found for Entry address" );
    });

    it("should fail to delete because wrong type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "delete_comment", {
		"id": post2.$addr,
	    });
	}, RibosomeError, "Failed to deserialize to entry type 'Comment'" );
    });

    it("should fail to delete because mismatched type", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( "happy_path", "happy_path", "delete_post", {
		"id": comment2.$id,
	    });
	}, RibosomeError, "Deserialized entry to wrong type: expected 0/0 but found 0/1" );
    });

    it("should fail to get because address is an 'update', not an 'origin' entry", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( "happy_path", "happy_path", "get_post", {
		"id": post2.$addr,
	    });
	}, RibosomeError, "is not a Create action type" );
    });
}

describe("DNArepo", () => {

    const holochain			= new Holochain();

    before(async function () {
	this.timeout( 30_000 );

	clients				= await backdrop( holochain, {
	    "happy_path":	DNA_PATH,
	}, [
	    "alice",
	]);
    });

    describe("Basic", basic_tests.bind( this, holochain ) );
    describe("Errors", errors_tests.bind( this, holochain ) );

    after(async () => {
	await holochain.stop();
	await holochain.destroy();
    });

});
