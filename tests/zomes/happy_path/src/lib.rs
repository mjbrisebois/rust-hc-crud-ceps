use hdk::prelude::*;
use hc_crud::{
    now,
    create_entity, get_entity, get_entities, update_entity, delete_entity,
    Entity, EntryModel, EntityType,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct GetEntityInput {
    pub id: EntryHash,
}

impl GetEntityInput {
    pub fn new(id: EntryHash) -> Self {
	GetEntityInput {
	    id: id,
	}
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEntityInput<T> {
    pub addr: ActionHash,
    pub properties: T,
}



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("Initialized 'happy_path' WASM");
    Ok(InitCallbackResult::Pass)
}


#[hdk_entry_helper]
#[derive(Clone)]
pub struct PostEntry {
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl EntryModel<EntryTypes> for PostEntry {
    fn name() -> &'static str { "Post" }
    fn get_type(&self) -> EntityType {
	EntityType::new( "post", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
	EntryTypes::Post(self.clone())
    }
}


#[hdk_entry_helper]
#[derive(Clone)]
pub struct CommentEntry {
    pub for_post: EntryHash,
    pub message: String,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
}

impl CommentEntry {
    pub fn to_input(&self) -> EntryTypes {
	EntryTypes::Comment(self.clone())
    }
}

impl EntryModel<EntryTypes> for CommentEntry {
    fn name() -> &'static str { "Comment" }
    fn get_type(&self) -> EntityType {
	EntityType::new( "comment", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
	EntryTypes::Comment(self.clone())
    }
}


#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    #[entry_def]
    Post(PostEntry),
    #[entry_def]
    Comment(CommentEntry),
}


#[hdk_link_types]
pub enum LinkTypes {
    Post,
    Comment,
}



// Post CRUD
#[hdk_extern]
pub fn create_post(mut post: PostEntry) -> ExternResult<Entity<PostEntry>> {
    if post.published_at.is_none() {
	post.published_at.replace( now()? );
    }

    debug!("Creating new post entry: {:?}", post );
    let entity = create_entity( &post )?;

    let pubkey = agent_info()?.agent_initial_pubkey;

    entity.link_from( &pubkey.into(), LinkTypes::Post, None )?;

    Ok( entity )
}



#[hdk_extern]
pub fn get_post(input: GetEntityInput) -> ExternResult<Entity<PostEntry>> {
    debug!("Get Post: {:?}", input.id );
    Ok( get_entity( &input.id )? )
}


#[hdk_extern]
pub fn update_post(mut input: UpdateEntityInput<PostEntry>) -> ExternResult<Entity<PostEntry>> {
    if input.properties.last_updated.is_none() {
	input.properties.last_updated.replace( now()? );
    }

    debug!("Updating post entry: {:?}", input.addr );
    let entity = update_entity( &input.addr, |previous: PostEntry, _| {
	let mut new_post = input.properties.clone();

	new_post.published_at = previous.published_at;

	Ok( new_post )
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn delete_post(input: GetEntityInput) -> ExternResult<ActionHash> {
    debug!("Get Post: {:?}", input.id );
    Ok( delete_entity::<PostEntry,EntryTypes>( &input.id )? )
}


// Comment CRUD
#[derive(Clone, Debug, Deserialize)]
pub struct CreateCommentInput {
    pub post_id: EntryHash,
    pub comment: CommentEntry,
}
#[hdk_extern]
pub fn create_comment(mut input: CreateCommentInput) -> ExternResult<Entity<CommentEntry>> {
    // Check that the post exists and is not deleted
    get_post( GetEntityInput::new( input.post_id.clone() ) )?;

    if input.comment.published_at.is_none() {
	input.comment.published_at.replace( now()? );
    }

    debug!("Creating new comment entry: {:?}", input.comment );
    let entity = create_entity( &input.comment )?;

    entity.link_from( &input.post_id, LinkTypes::Comment, None )?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_comment(input: GetEntityInput) -> ExternResult<Entity<CommentEntry>> {
    debug!("Get Post: {:?}", input.id );
    Ok(
	get_entity( &input.id )?
    )
}


#[hdk_extern]
pub fn get_comments_for_post(post_id: EntryHash) -> ExternResult<Vec<Entity<CommentEntry>>> {
    Ok( get_entities( &post_id, LinkTypes::Comment, None )? )
}


#[hdk_extern]
pub fn update_comment(mut input: UpdateEntityInput<CommentEntry>) -> ExternResult<Entity<CommentEntry>> {
    if input.properties.last_updated.is_none() {
	input.properties.last_updated.replace( now()? );
    }

    debug!("Updating comment entry: {:?}", input.addr );
    let entity = update_entity( &input.addr, |previous: CommentEntry, _| {
	let mut new_comment = input.properties.clone();

	new_comment.published_at = previous.published_at;

	Ok( new_comment )
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn delete_comment(input: GetEntityInput) -> ExternResult<ActionHash> {
    debug!("Get Comment: {:?}", input.id );
    Ok( delete_entity::<CommentEntry,EntryTypes>( &input.id )? )
}


#[derive(Clone, Debug, Deserialize)]
pub struct LinkCommentToPostInput {
    pub comment_id: EntryHash,
    pub post_id: EntryHash,
}
#[hdk_extern]
pub fn link_comment_to_post (input: LinkCommentToPostInput) -> ExternResult<ActionHash> {
    Ok( create_link(
	input.post_id,
	input.comment_id,
	LinkTypes::Comment,
	()
    )? )
}


#[derive(Clone, Debug, Deserialize)]
pub struct MoveCommentInput {
    pub comment_addr: ActionHash,
    pub post_id: EntryHash,
}
#[hdk_extern]
pub fn move_comment_to_post (input: MoveCommentInput) -> ExternResult<Entity<CommentEntry>> {
    let mut current_base = input.post_id.clone();
    let new_base = input.post_id.clone();

    let entity = update_entity( &input.comment_addr, |mut previous: CommentEntry, _| {
	current_base = previous.for_post;
	previous.for_post = new_base.to_owned();

	Ok( previous )
    })?;

    debug!("Delinking previous base to ENTRY: {:?}", current_base );
    entity.move_link_from( LinkTypes::Comment, None, &current_base, &new_base )?;

    Ok( entity )
}
