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
let create_comment_input		= {
    "message": "Don't call me surely",
};


function basic_tests () {
    it("should test 'create_entity'", async function () {
	post				= await clients.alice.callEntity( "happy_path", "happy_path", "create_post", create_post_input );

	expect( post.message		).to.equal( create_post_input.message );

	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "create_post", create_post_input );
    });

    it("should test 'get_entity'", async function () {
	post				= await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
	    "id": post.$id,
	});

	expect( post.message		).to.equal( create_post_input.message );
    });

    it("should test 'update_entity'", async function () {
	let input			= Object.assign( {}, create_post_input, {
	    "message": "Goodbye, world!",
	});

	let prev_post			= post2;
	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "update_post", {
	    "addr": post2.$addr,
	    "properties": input,
	});

	expect( post2.message		).to.equal( input.message );
	expect( post2.$header		).to.not.deep.equal( prev_post.$header );

	post2				= await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
	    "id": post2.$id,
	});

	expect( post2.message		).to.equal( input.message );
	expect( post2.$header		).to.not.deep.equal( prev_post.$header );
    });

    it("should test 'Collection'", async function () {
	this.timeout( 5_000 );
	{
	    create_comment_input.for_post = post.$id;
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post.$id,
		"comment": create_comment_input,
	    });

	    expect( comment.message		).to.equal( create_comment_input.message );
	    expect( comment.for_post.$id	).to.deep.equal( post.$id );

	    create_comment_input.for_post = post2.$id;
	    comment2			= await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post2.$id,
		"comment": create_comment_input,
	    });
	}

	{
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "get_comment", {
		"id": comment.$id,
	    });

	    expect( comment.message		).to.equal( create_comment_input.message );
	    expect( comment.for_post.$id	).to.deep.equal( post.$id );
	}

	{
	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 1 );
	}

	{
	    let input			= Object.assign( {}, create_comment_input, {
		"message": "I just want to tell you both, good luck. We're all counting on you.",
	    });

	    let prev_comment		= comment;
	    comment			= await clients.alice.callEntity( "happy_path", "happy_path", "update_comment", {
		"addr": comment.$addr,
		"properties": input,
	    });

	    expect( comment.$header	).to.not.deep.equal( prev_comment.$header );

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 1 );
	    expect( comments[0].message	).to.equal( input.message );
	    expect( comments[0].$header	).to.not.deep.equal( prev_comment.$header );
	}

	{
	    let delete_hash		= await clients.alice.call( "happy_path", "happy_path", "delete_comment", {
		"id": comment.$id,
	    });

	    let comments		= await clients.alice.callCollection( "happy_path", "happy_path", "get_comments_for_post", post.$id );

	    expect( comments		).to.have.length( 0 );
	}
    });

    it("should test 'delete_entity'", async function () {
	let delete_hash			= await clients.alice.call( "happy_path", "happy_path", "delete_post", {
	    "id": post.$id,
	});

	expect( delete_hash		).to.be.a("HeaderHash");
    });
}

function errors_tests () {
    it("should fail to 'get_entity' because address is wrong entry type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "get_post", {
		"id": comment2.$id,
	    });
	}, RibosomeError, "Deserialized entry to wrong type" );
    });

    it("should fail to update because of wrong entry type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "update_comment", {
		"addr": post2.$addr,
		"properties": create_comment_input,
	    });
	}, RibosomeError, "Failed to deserialize entry to type" );
    });

    it("should fail to update because mismatched type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "update_post", {
		"addr": comment2.$addr,
		"properties": create_post_input,
	    });
	}, RibosomeError, "Deserialized entry to wrong type" );
    });

    it("should fail to create comment because post is deleted", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "create_comment", {
		"post_id": post.$id,
		"comment": create_comment_input,
	    });
	}, RibosomeError, "Entry not found for address" );
    });

    it("should fail to delete because wrong type", async function () {
	await expect_reject( async () => {
	    await clients.alice.callEntity( "happy_path", "happy_path", "delete_comment", {
		"id": post2.$addr,
	    });
	}, RibosomeError, "Failed to deserialize entry to type" );
    });

    it("should fail to delete because mismatched type", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( "happy_path", "happy_path", "delete_post", {
		"id": comment2.$id,
	    });
	}, RibosomeError, "Deserialized entry to wrong type" );
    });

    it("should fail to get because address is an 'update', not an 'origin' entry", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( "happy_path", "happy_path", "get_post", {
		"id": post2.$addr,
	    });
	}, RibosomeError, "is an 'update'; Use origin address" );
    });

    it("should fail to get because ", async function () {
	await expect_reject( async () => {
	    await clients.alice.call( "happy_path", "happy_path", "get_posts_for_comment", post2.$id );
	}, RibosomeError, "is not the expected type: App" );
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
