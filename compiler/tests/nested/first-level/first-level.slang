entity User {
    /** The ability to reaaad */
    write.basic;
    read;

    company
        :authorization
        :user-type
        -> Company;
    sibling -> User;
}

@User {
    sibling {
        read;
    }
}