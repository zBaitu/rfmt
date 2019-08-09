type ProposeTransactionsFuture: Future<
            Item = ProposeTransactionsResponse<Self::MessageId>,
            Error = Error,
        > + Send
        + 'static;
