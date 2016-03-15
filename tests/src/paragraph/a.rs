fn trans_comments(cmnts: Vec<rst::Comment>) -> Vec<Comment> {
    let mut pre_blank_line_pos = 0;
    let mut blank_line = 0;

    cmnts.into_iter().fold(Vec::new(), |mut cmnts, cmnt| {
        if cmnt.style == rst::CommentStyle::BlankLine {
            let cur_pos = cmnt.pos.0;

            if cur_pos != pre_blank_line_pos + 1 {
                blank_line = 1;
                cmnts.push(trans_comment(cmnt));
            } else {
                blank_line += 1;
                if blank_line <= MAX_BLANK_LINE {
                    cmnts.push(trans_comment(cmnt));
                }
            }

            pre_blank_line_pos = cur_pos;
        } else {
            blank_line = 0;
            cmnts.push(trans_comment(cmnt));
        }

        cmnts
    })
}
