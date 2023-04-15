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


module.exports = {
    schema,
};
